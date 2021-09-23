use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::marker::PhantomData;
use std::process::id;
use std::sync::Arc;
use std::time::Instant;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, Lens, LensExt, Rect, SingleUse};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};
use druid_widget_nursery::selectors;

use crate::widgets::header::{Header, HeaderConstrains};
use crate::widgets::tree::node::{
    OpenerFactory, TreeItemFactory, TreeNode, TreeNodeWidget, TREE_CHILD_REMOVE_INTERNAL,
    TREE_CHILD_SHOW, TREE_NODE_REMOVE, TREE_OPEN,
};
use crate::widgets::tree::NodeIndex;

macro_rules! impl_simple_tree_root {
    ($root_name:ident, $node_name:ident) => {
        impl TreeNodeRoot<$node_name> for $root_name {
            fn children_count(&self) -> usize {
                self.children.len()
            }

            fn get_child(&self, index: usize) -> &$node_name {
                &self.children[index]
            }

            fn for_child_mut<V>(&mut self, index: usize, cb: impl FnOnce(&mut $node_name, usize) -> V) -> V {
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
pub(crate) use impl_simple_tree_root;


pub trait TreeNodeRoot<T: TreeNode>
where
    Self: Data + std::fmt::Debug,
{
    /// Returns how many children are below this node. It could be zero if this is a leaf.
    fn children_count(&self) -> usize;

    /// Returns a reference to the node's child at the given index
    fn get_child(&self, index: usize) -> &T;

    /// Returns a mutable reference to the node's child at the given index
    fn for_child_mut<V>(&mut self, index: usize, cb: impl FnOnce(&mut T, usize) -> V) -> V;

    /// Remove the child at `index`
    fn rm_child(&mut self, index: usize);

    fn node(&self, idx: &[usize]) -> &T {
        if idx.len() == 0 {
            panic!("Can't access root node")
        } else {
            self.get_child(idx[0]).node(&idx[1..])
        }
    }

    fn node_mut<V>(&mut self, idx: &[usize], cb: impl FnOnce(&mut T, usize) -> V) -> V {
        match idx.len() {
            0 => panic!("can't access root"),
            1 => self.for_child_mut(idx[0], cb),
            _ => self.for_child_mut(idx[0], move |child, _| child.node_mut(&idx[1..], cb)),
        }
    }
}

pub struct TreeNodeRootWidget<R, T, L, const N: usize>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool>,
{
    /// The label for this node
    constrains: HeaderConstrains<N>,
    /// The children of this tree node widget
    children: Vec<WidgetPod<T, TreeNodeWidget<T, L, N>>>,
    /// A factory closure for the user defined widget
    make_widget: TreeItemFactory<T, N>,
    /// A factory closure for the user defined opener
    make_opener: Arc<Box<OpenerFactory<T>>>,
    /// The user must provide a Lens<T, bool> that tells if
    /// the node is expanded or not.
    expand_lens: L,
    _marker: PhantomData<R>,
}

impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone, const N: usize>
    TreeNodeRootWidget<R, T, L, N>
{
    //     /// Create a TreeNodeWidget from a TreeNode.
    pub fn new(
        make_widget: TreeItemFactory<T, N>,
        make_opener: Arc<Box<OpenerFactory<T>>>,
        constrains: HeaderConstrains<N>,
        expand_lens: L, // expanded: bool,
    ) -> Self {
        Self {
            constrains,
            // expanded,
            children: Vec::new(),
            make_widget,
            make_opener,
            expand_lens,
            _marker: PhantomData,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    pub fn update_children(&mut self, data: &R) -> bool {
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
                    0,
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
        changed
    }

    pub fn remove_highlights(&mut self) -> bool {
        let mut changed = false;
        for child in self.children.iter_mut() {
            changed |= child.widget_mut().remove_highlights()
        }
        changed
    }

    pub fn get_selected(&self) -> Vec<NodeIndex> {
        let mut r = Vec::new();
        for (i, child) in self.children.iter().enumerate() {
            child.widget().get_selected(&mut r, vec![i]);
        }
        r
    }

    pub fn at(&self, p: Point) -> Option<NodeIndex> {
        for child in &self.children {
            let rect = child.layout_rect();
            if rect.contains(p) {
                let mut r = Vec::new();
                child
                    .widget()
                    .at(Point::new(p.x - rect.x0, p.y - rect.y0), &mut r);
                return Some(r);
            }
        }
        None
    }

    pub fn node(&self, idx: &[usize]) -> &TreeNodeWidget<T, L, N> {
        if idx.len() == 0 {
            panic!("Empty idx")
        } else {
            self.children[idx[0]].widget().node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut TreeNodeWidget<T, L, N> {
        if idx.len() == 0 {
            panic!("Empty idx")
        } else {
            self.children[idx[0]].widget_mut().node_mut(&idx[1..])
        }
    }
}

impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> Widget<R>
    for TreeNodeRootWidget<R, T, L, N>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        match event {
            Event::Notification(notif) if notif.is(TREE_OPEN) => {
                panic!("should not happen")
            }
            Event::Notification(notif) if notif.is(TREE_NODE_REMOVE) => {
                panic!("should not happen")
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
            // TODO?
            // Event::Notification(notif) if notif.is(TREE_NOTIFY_PARENT) => {
            //     if self.widget.id() != notif.source() {
            //         let notif = notif.get(TREE_NOTIFY_PARENT).unwrap();
            //         ctx.submit_command(TREE_NOTIFY_PARENT.with(*notif).to(self.widget.id()));
            //         ctx.set_handled();
            //     }
            //     None
            // }
            _ => (),
        };

        // don't go further with unhandled notifications
        if let Event::Notification(_) = event {
            return;
        }

        // send the event to the opener if the widget is visible or the event also targets
        // hidden widgets.

        // Forward to children nodes
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            data.for_child_mut(index, |data: &mut T, _index: usize| {
                if child_widget_node.is_initialized() {
                    child_widget_node.event(ctx, event, data, env)
                }
            });
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &R, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_children(data) {
                ctx.children_changed();
            }
        }
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.lifecycle(ctx, event, child_tree_node, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &R, data: &R, env: &Env) {
        if self.update_children(data) {
            for child_widget_node in self.children.iter_mut() {
                // TODO: this is not true except for the new child. `update_children` should tell
                // which child was added/removed...
                ctx.submit_command(TREE_CHILD_SHOW.to(child_widget_node.id()))
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

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &R, env: &Env) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let max_width = self.constrains.max_width;
        let mut current_height = 0.;
        let mut paint_rect = Rect::ZERO;

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

        let my_size = Size::new(max_width, current_height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.paint(ctx, child_tree_node, env);
        }
    }
}
