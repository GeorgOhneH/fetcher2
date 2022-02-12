use druid::{Lens, LensExt, Rect, theme};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetPod,
};
use druid::kurbo::Size;
use druid::piet::RenderContext;
use druid_widget_nursery::selectors;
pub(crate) use impl_simple_tree_node;
use std::convert::TryFrom;
use std::sync::Arc;

use crate::widgets::tree::header::HeaderConstrains;
use crate::widgets::tree::NodeIndex;
use crate::widgets::tree::opener::Opener;

selectors! {
    /// Notification to send from the widget that requires removal
    TREE_NODE_REMOVE,
    /// Internal use, sent by TreeNodeWidgets with their index for their parent TreeNodeWidget
    /// TODO: should not be public
    TREE_CHILD_REMOVE_INTERNAL: i32,
    /// Notification that opens the first encountered branch node.
    TREE_OPEN,
    /// Command sent to children on open
    TREE_CHILD_SHOW,
    /// Command sent to children on close
    TREE_CHILD_HIDE,
    /// Notify an opener's widget on click.
    TREE_ACTIVATE_NODE,
}

macro_rules! impl_simple_tree_node {
    ($node_name:ident) => {
        impl TreeNode for $node_name {
            fn children_count(&self) -> usize {
                self.children.len()
            }

            fn get_child(&self, index: usize) -> &Self {
                &self.children[index]
            }

            fn for_child_mut<V>(
                &mut self,
                index: usize,
                cb: impl FnOnce(&mut Self, usize) -> V,
            ) -> V {
                let mut new_child = self.children[index].to_owned();
                let v = cb(&mut new_child, index);
                if !new_child.same(&self.children[index]) {
                    self.children[index] = new_child;
                }
                v
            }

            fn rm_child(&mut self, index: usize) {
                self.children.remove(index);
            }
        }
    };
}
/// A tree node `Data`. This is the data expected by the tree widget.
///
/// Implementors of this trait must know the number of children of each node
/// and be able to provide a children based on the index of the child widget.
/// This implies that the implementation of the collection of children may be
/// abstracted away in the data as long as `children_count()`, `get_child()`,
/// rm_child() and for_child_mut()` accessors give coherent results. This is
/// a way to implement filtering and sorting at the app data level.
pub trait TreeNode
where
    Self: Data + std::fmt::Debug,
{
    /// Returns how many children are below this node. It could be zero if this is a leaf.
    fn children_count(&self) -> usize;

    /// Returns a reference to the node's child at the given index
    fn get_child(&self, index: usize) -> &Self;

    /// Returns a mutable reference to the node's child at the given index
    fn for_child_mut<V>(&mut self, index: usize, cb: impl FnOnce(&mut Self, usize) -> V) -> V;

    /// `is_branch` must return `true` if the data is considered as a branch.
    /// The default implementation returns `true` when `children_count()` is
    /// more than 0.
    fn is_branch(&self) -> bool {
        self.children_count() > 0
    }

    /// Remove the child at `index`
    fn rm_child(&mut self, index: usize);

    fn node(&self, idx: &[usize]) -> &Self {
        if idx.is_empty() {
            self
        } else {
            self.get_child(idx[0]).node(&idx[1..])
        }
    }

    fn node_mut<V>(&mut self, idx: &[usize], cb: impl FnOnce(&mut Self, usize) -> V) -> V {
        match idx.len() {
            0 => unreachable!(),
            1 => self.for_child_mut(idx[0], cb),
            _ => self.for_child_mut(idx[0], move |child, _| child.node_mut(&idx[1..], cb)),
        }
    }
}

pub type TreeItemFactory<T, const N: usize> = [Arc<dyn Fn() -> Box<dyn Widget<T>>>; N];
pub type OpenerFactory<T> = dyn Fn() -> Box<dyn Widget<T>>;

/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
pub struct TreeNodeWidget<T, L, const N: usize>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    // the index of the widget in its parent
    pub index: usize,
    // The "opener" widget,
    pub opener: WidgetPod<T, Opener<T>>,
    /// The label for this node
    pub widgets: [WidgetPod<T, Box<dyn Widget<T>>>; N],
    pub constrains: HeaderConstrains<N>,
    pub depth: usize,
    /// The children of this tree node widget
    pub children: Vec<WidgetPod<T, Self>>,
    /// A factory closure for the user defined widget
    pub make_widget: TreeItemFactory<T, N>,
    /// A factory closure for the user defined opener
    pub make_opener: Arc<OpenerFactory<T>>,
    /// The user must provide a Lens<T, bool> that tells if
    /// the node is expanded or not.
    pub expand_lens: L,

    pub is_content_hover: bool,
    pub selected: bool,
}

impl<T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> TreeNodeWidget<T, L, N> {
    /// Create a TreeNodeWidget from a TreeNode.
    pub fn new(
        make_widget: TreeItemFactory<T, N>,
        make_opener: Arc<OpenerFactory<T>>,
        index: usize,
        constrains: HeaderConstrains<N>,
        depth: usize,
        expand_lens: L, // expanded: bool,
    ) -> Self {
        let widgets = make_widget
            .clone()
            .map(|widget_fn| WidgetPod::new((widget_fn)()));
        Self {
            index,
            opener: WidgetPod::new(Opener::new(make_opener.clone()())),
            widgets,
            constrains,
            depth,
            // expanded,
            children: Vec::new(),
            make_widget,
            make_opener,
            expand_lens,
            is_content_hover: false,
            selected: false,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    pub fn update_children(&mut self, data: &T) -> bool {
        let mut changed = false;
        if self.children.len() > data.children_count() {
            self.children.truncate(data.children_count());
            changed = true;
        }
        for index in 0..data.children_count() {
            changed |= index >= self.children.len();
            match self.children.get_mut(index) {
                Some(c) => c.widget_mut().index = index,
                None => self.children.push(WidgetPod::new(TreeNodeWidget::new(
                    self.make_widget.clone(),
                    self.make_opener.clone(),
                    index,
                    self.constrains.clone(),
                    self.depth + 1,
                    self.expand_lens.clone(),
                ))),
            }
        }
        changed
    }

    pub fn update_constrains(&mut self, constrains: HeaderConstrains<N>) {
        for child in self.children.iter_mut() {
            child.widget_mut().update_constrains(constrains.clone())
        }
        self.constrains = constrains
    }

    pub fn update_highlights(&mut self, p: Point) -> bool {
        let mut changed = false;
        for child in self.children.iter_mut() {
            let rect = child.layout_rect();
            changed |= if rect.contains(p) {
                child
                    .widget_mut()
                    .update_highlights(Point::new(p.x - rect.x0, p.y - rect.y0))
            } else {
                child.widget_mut().remove_highlights()
            }
        }
        let new = self.hit_content_area(p);
        changed |= self.is_content_hover != new;
        self.is_content_hover = self.hit_content_area(p);
        changed
    }

    pub fn remove_highlights(&mut self) -> bool {
        let mut changed = false;
        for child in self.children.iter_mut() {
            changed |= child.widget_mut().remove_highlights()
        }
        changed |= self.is_content_hover;
        self.is_content_hover = false;
        changed
    }

    pub fn get_selected(&self, selected: &mut Vec<NodeIndex>, current_idx: NodeIndex) {
        for (i, child) in self.children.iter().enumerate() {
            let mut idx = current_idx.clone();
            idx.push(i);
            child.widget().get_selected(selected, idx)
        }
        if self.selected {
            selected.push(current_idx)
        }
    }

    pub fn at(&self, p: Point, idx: &mut Vec<usize>) {
        idx.push(self.index);
        for child in &self.children {
            let rect = child.layout_rect();
            if rect.contains(p) {
                child
                    .widget()
                    .at(Point::new(p.x - rect.x0, p.y - rect.y0), idx);
                return;
            }
        }
    }

    pub fn node(&self, idx: &[usize]) -> &Self {
        if idx.is_empty() {
            self
        } else {
            self.children[idx[0]].widget().node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut Self {
        if idx.is_empty() {
            self
        } else {
            self.children[idx[0]].widget_mut().node_mut(&idx[1..])
        }
    }

    fn hit_content_area(&self, p: Point) -> bool {
        let width = self.constrains.max_width;
        let mut height = self.opener.layout_rect().height();
        let origin_x = self.opener.layout_rect().x1;
        for widget in &self.widgets {
            height = height.max(widget.layout_rect().height());
        }
        Rect::new(origin_x, 0., width, height).contains(p)
    }
}

impl<T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> Widget<T> for TreeNodeWidget<T, L, N> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Notification(notif) if notif.is(TREE_OPEN) => {
                if data.is_branch() {
                    ctx.set_handled();
                    if !self.expand_lens.get(data) {
                        self.expand_lens.put(data, true);
                        if self.update_children(data) {
                            ctx.children_changed();
                        }
                        for child_widget_node in self.children.iter_mut() {
                            ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                        }
                    }
                    return;
                }
            }
            Event::Notification(notif) if notif.is(TREE_NODE_REMOVE) => {
                // we were commanded to remove ourselves. Let's tell our parent.
                ctx.submit_notification(TREE_CHILD_REMOVE_INTERNAL.with(self.index as i32));
                ctx.set_handled();
                return;
            }
            Event::Notification(notif) if notif.is(TREE_CHILD_REMOVE_INTERNAL) => {
                // get the index to remove from the notification
                let index =
                    usize::try_from(*notif.get(TREE_CHILD_REMOVE_INTERNAL).unwrap()).unwrap();
                // remove the widget and the data
                self.children.remove(index);
                data.rm_child(index);
                // update our children
                self.update_children(data);
                ctx.set_handled();
                ctx.children_changed();
                return;
            }
            _ => (),
        };

        // don't go further with unhandled notifications
        if let Event::Notification(_) = event {
            println!("SHould this ever happen?");
            return;
        }

        self.widgets
            .iter_mut()
            .for_each(|widget| widget.event(ctx, event, data, env));

        if data.is_branch() {
            // send the event to the opener if the widget is visible or the event also targets
            // hidden widgets.
            let before = self.expand_lens.get(data);
            self.opener.event(ctx, event, data, env);
            let expanded = self.expand_lens.get(data);

            if expanded != before {
                // The opener widget has decided to change the expanded/collapsed state of the node,
                // handle it by expanding/collapsing children nodes as required.

                let cmd: Selector;
                if expanded {
                    cmd = TREE_CHILD_SHOW;
                    // create child widgets if needed.
                    if self.update_children(data) {
                        // New children were created, inform the context.
                        ctx.children_changed();
                    }
                } else {
                    cmd = TREE_CHILD_HIDE;
                    // self.children = vec![];
                };
                for child_widget_node in self.children.iter_mut() {
                    ctx.submit_command(cmd.to(child_widget_node.id()))
                }
                ctx.request_layout();
            }
            // Forward to children nodes
            if event.should_propagate_to_hidden() {
                // forward unconditionally
                for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                    data.for_child_mut(index, |data: &mut T, _index: usize| {
                        if child_widget_node.is_initialized() {
                            child_widget_node.event(ctx, event, data, env)
                        }
                    });
                }
            } else if expanded & before {
                for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                    data.for_child_mut(index, |data: &mut T, _index: usize| {
                        child_widget_node.event(ctx, event, data, env)
                    });
                }
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.expand_lens.get(data) && self.update_children(data) {
                ctx.children_changed();
            }
        }
        self.opener.lifecycle(ctx, event, data, env);
        self.widgets
            .iter_mut()
            .for_each(|widget| widget.lifecycle(ctx, event, data, env));
        if data.is_branch() & (event.should_propagate_to_hidden() | self.expand_lens.get(data)) {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.lifecycle(ctx, event, child_tree_node, env);
            }
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widgets
            .iter_mut()
            .for_each(|widget| widget.update(ctx, data, env));
        self.opener.update(ctx, data, env);
        if self.update_children(data) {
            if self.expand_lens.get(data) {
                for child_widget_node in self.children.iter_mut() {
                    // TODO: this is not true except for the new child. `update_children` should tell
                    // which child was added/removed...
                    ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
                }
            }
            ctx.children_changed();
        }

        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            if child_widget_node.is_initialized() {
                let child_tree_node = data.get_child(index);
                child_widget_node.update(ctx, child_tree_node, env);
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let indent = env.get(theme::BASIC_WIDGET_HEIGHT);
        let max_width = self.constrains.max_width;

        let opener_widget_x = indent * self.depth as f64 + basic_size;
        let mut current_height = basic_size;
        let mut paint_rect = Rect::ZERO;

        // Immediately on the right, the node widget
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            let (left_width, right_width) = self.constrains.sizes[idx];
            let widget_width = if idx == 0 {
                (right_width - left_width - opener_widget_x).max(0.)
            } else {
                right_width - left_width
            };
            let widget_bc = BoxConstraints::new(
                Size::new(widget_width, basic_size),
                Size::new(widget_width, f64::INFINITY),
            );
            let widget_size = widget.layout(ctx, &widget_bc, data, env);
            current_height = current_height.max(widget_size.height);
            let widget_pos_x = if idx == 0 {
                left_width + opener_widget_x
            } else {
                left_width
            };
            let widget_pos = Point::new(widget_pos_x, 0.);
            widget.set_origin(ctx, data, env, widget_pos);
            paint_rect = paint_rect.union(widget.paint_rect());
        }

        // Top left, the opener
        self.opener.layout(
            ctx,
            &BoxConstraints::tight(Size::new(basic_size, basic_size)),
            data,
            env,
        );
        self.opener.set_origin(
            ctx,
            data,
            env,
            Point::new(
                indent * self.depth as f64,
                (current_height - basic_size).max(0.) / 2.,
            ),
        );

        if self.expand_lens.get(data) {
            for (idx, child) in self.children.iter_mut().enumerate() {
                let child_bc = BoxConstraints::new(
                    Size::new(max_width, basic_size),
                    Size::new(max_width, f64::INFINITY),
                );
                let child_data = data.get_child(idx);
                let child_size = child.layout(ctx, &child_bc, child_data, env);
                let child_pos = Point::new(0., current_height);
                child.set_origin(ctx, child_data, env, child_pos);
                paint_rect = paint_rect.union(child.paint_rect());
                current_height += child_size.height;
            }
        }

        let my_size = Size::new(max_width, current_height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let width = self.constrains.max_width;
        let mut height = self.opener.layout_rect().height();
        let origin_x = self.opener.layout_rect().x1;
        for widget in &self.widgets {
            height = height.max(widget.layout_rect().height());
        }
        let _background_rect = Rect::new(0., 0., width, height);
        let highlight_rect = Rect::new(origin_x, 0., width, height);
        if self.selected {
            ctx.fill(highlight_rect, &env.get(theme::PRIMARY_DARK))
        } else if self.is_content_hover {
            ctx.fill(highlight_rect, &env.get(theme::PRIMARY_LIGHT))
        }

        self.opener.paint(ctx, data, env);
        for widget in self.widgets.iter_mut() {
            ctx.render_ctx.save().unwrap();
            ctx.clip(widget.layout_rect());
            widget.paint(ctx, data, env);
            ctx.render_ctx.restore().unwrap();
        }
        if data.is_branch() & self.expand_lens.get(data) {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
    }
}
