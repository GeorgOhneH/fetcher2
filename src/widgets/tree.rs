use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, Lens, LensExt, Rect};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};

use crate::widgets::header::{Header, HEADER_SIZE_CHANGED};
use druid_widget_nursery::selectors;
use std::process::id;

// TODO:
//   - TREE_CLOSE command that mirrors TreeOpen
//   - TREE_OPEN_ALL command to open recursively
//   - TREE_CLOSE_ALL command to close recursively
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
    /// Submitted as a notification, from the user's widget, the Selector payload is submitted as
    /// a command to its parent. It's a workaround to simulate notifications between user's tree
    /// widgets.
    TREE_NOTIFY_PARENT: Selector,
    /// Notify an opener's widget on click.
    TREE_ACTIVATE_NODE,
}

/// A tree widget for a collection of items organized in a hierarchical way.
pub struct Tree<R, T, L, const N: usize>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool>,
{
    header: WidgetPod<R, Header<R, N>>,
    /// The root node of this tree
    root_node: WidgetPod<R, TreeNodeRootWidget<R, T, L, N>>,
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
    fn for_child_mut(&mut self, index: usize, cb: impl FnMut(&mut Self, usize));

    /// `is_branch` must return `true` if the data is considered as a branch.
    /// The default implementation returns `true` when `children_count()` is
    /// more than 0.
    fn is_branch(&self) -> bool {
        self.children_count() > 0
    }

    /// Remove the child at `index`
    fn rm_child(&mut self, index: usize);
}

pub trait TreeNodeRoot<T: TreeNode>
where
    Self: Data + std::fmt::Debug,
{
    /// Returns how many children are below this node. It could be zero if this is a leaf.
    fn children_count(&self) -> usize;

    /// Returns a reference to the node's child at the given index
    fn get_child(&self, index: usize) -> &T;

    /// Returns a mutable reference to the node's child at the given index
    fn for_child_mut(&mut self, index: usize, cb: impl FnMut(&mut T, usize));

    /// Remove the child at `index`
    fn rm_child(&mut self, index: usize);
}

// Wrapper widget that reacts to clicks by sending a TREE_ACTIVATE_NODE command to
// its inner user-defined widget.
// TODO: Try use a Controller instead of a plain widget.
struct Opener<T>
where
    T: TreeNode,
{
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
}

/// Implementing Widget for the Opener.
impl<T: TreeNode> Widget<T> for Opener<T>
where
    T: Data,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_command(TREE_ACTIVATE_NODE.to(self.widget.id()));
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        self.widget.event(ctx, event, data, _env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.widget.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.widget.layout(ctx, bc, data, env);
        self.widget.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

// The default opener if none is passed to the Tree builder.
struct Wedge<T, L>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    expand_lens: L,
    phantom: PhantomData<T>,
}

impl<T: TreeNode, L: Lens<T, bool>> Widget<T> for Wedge<T, L> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(TREE_ACTIVATE_NODE) => {
                self.expand_lens.put(data, !self.expand_lens.get(data));
                ctx.set_handled();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if !data.is_branch() {
            return;
        }
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the opener
        let mut path = BezPath::new();
        if self.expand_lens.get(data) {
            // expanded: 'V' shape
            path.move_to((5.0, 7.0));
            path.line_to((9.0, 13.0));
            path.line_to((13.0, 7.0));
        } else {
            // collapsed: '>' shape
            path.move_to((7.0, 5.0));
            path.line_to((13.0, 9.0));
            path.line_to((7.0, 13.0));
        }
        let style = StrokeStyle::new()
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        ctx.stroke_styled(path, &stroke_color, 2.5, &style);
    }
}

type TreeItemFactory<T, const N: usize> = [Arc<dyn Fn() -> Box<dyn Widget<T>>>; N];
type OpenerFactory<T> = dyn Fn() -> Box<dyn Widget<T>>;

fn make_wedge<T: TreeNode, L: Lens<T, bool>>(expand_lens: L) -> Wedge<T, L> {
    Wedge {
        phantom: PhantomData,
        expand_lens,
    }
}

/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
struct TreeNodeWidget<T, L, const N: usize>
where
    T: TreeNode,
    L: Lens<T, bool>,
{
    // the index of the widget in its parent
    index: usize,
    // The "opener" widget,
    opener: WidgetPod<T, Opener<T>>,
    /// The label for this node
    widgets: [WidgetPod<T, Box<dyn Widget<T>>>; N],
    sizes: [(f64, f64); N],
    depth: usize,
    /// The children of this tree node widget
    children: Vec<WidgetPod<T, Self>>,
    /// A factory closure for the user defined widget
    make_widget: TreeItemFactory<T, N>,
    /// A factory closure for the user defined opener
    make_opener: Arc<Box<OpenerFactory<T>>>,
    /// The user must provide a Lens<T, bool> that tells if
    /// the node is expanded or not.
    expand_lens: L,
}

impl<T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> TreeNodeWidget<T, L, N> {
    /// Create a TreeNodeWidget from a TreeNode.
    fn new(
        make_widget: TreeItemFactory<T, N>,
        make_opener: Arc<Box<OpenerFactory<T>>>,
        index: usize,
        sizes: [(f64, f64); N],
        depth: usize,
        expand_lens: L, // expanded: bool,
    ) -> Self {
        let widgets = make_widget
            .clone()
            .map(|widget_fn| WidgetPod::new((widget_fn)()));
        Self {
            index,
            opener: WidgetPod::new(Opener {
                widget: WidgetPod::new(make_opener.clone()()),
            }),
            widgets,
            sizes,
            depth,
            // expanded,
            children: Vec::new(),
            make_widget,
            make_opener,
            expand_lens,
        }
    }

    /// Expand or collapse the node.
    /// Returns whether new children were created.
    fn update_children(&mut self, data: &T) -> bool {
        let mut changed = false;
        if self.expand_lens.get(data) {
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
                        self.sizes.clone(),
                        self.depth + 1,
                        self.expand_lens.clone(),
                    ))),
                }
            }
        }
        changed
    }

    fn update_sizes(&mut self, sizes: [(f64, f64); N]) {
        for child in self.children.iter_mut() {
            child.widget_mut().update_sizes(sizes)
        }
        self.sizes = sizes
    }
}

impl<T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> Widget<T> for TreeNodeWidget<T, L, N> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // match event {
        //     Event::MouseMove(_) => (),
        //     _ => eprintln!("{:?} {:?}", ctx.widget_id(), event),
        // }
        let event = match event {
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
                    None
                } else {
                    Some(event)
                }
            }
            Event::Notification(notif) if notif.is(TREE_NODE_REMOVE) => {
                // we were commanded to remove ourselves. Let's tell our parent.
                ctx.submit_notification(TREE_CHILD_REMOVE_INTERNAL.with(self.index as i32));
                ctx.set_handled();
                None
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
                None
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
            _ => Some(event),
        };

        // get the unhandled event or return
        let event = if let Some(evt) = event {
            evt
        } else {
            return;
        };

        // don't go further with unhandled notifications
        if let Event::Notification(_) = event {
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
            if self.update_children(data) {
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
        let width = bc.max().width;

        let current_width = indent * self.depth as f64 + basic_size;
        let mut current_height = basic_size;
        let mut paint_rect = Rect::ZERO;

        // Immediately on the right, the node widget
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            let (left_width, right_width) = self.sizes[idx];
            let widget_width = if idx == 0 {
                (right_width - left_width - current_width).max(0.)
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
                left_width + current_width
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
                    Size::new(width, basic_size),
                    Size::new(width, f64::INFINITY),
                );
                let child_data = data.get_child(idx);
                let child_size = child.layout(ctx, &child_bc, child_data, env);
                let child_pos = Point::new(0., current_height);
                child.set_origin(ctx, child_data, env, child_pos);
                paint_rect = paint_rect.union(child.paint_rect());
                current_height += child_size.height;
            }
        }

        let my_size = Size::new(width, current_height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.opener.paint(ctx, data, env);
        for widget in self.widgets.iter_mut() {
            widget.paint(ctx, data, env);
        }
        if data.is_branch() & self.expand_lens.get(data) {
            for (index, child_widget_node) in self.children.iter_mut().enumerate() {
                let child_tree_node = data.get_child(index);
                child_widget_node.paint(ctx, child_tree_node, env);
            }
        }
    }
}

struct TreeNodeRootWidget<R, T, L, const N: usize>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool>,
{
    /// The label for this node
    sizes: [(f64, f64); N],
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
    fn new(
        make_widget: TreeItemFactory<T, N>,
        make_opener: Arc<Box<OpenerFactory<T>>>,
        sizes: [(f64, f64); N],
        expand_lens: L, // expanded: bool,
    ) -> Self {
        Self {
            sizes,
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
    fn update_children(&mut self, data: &R) -> bool {
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
                    self.sizes.clone(),
                    0,
                    self.expand_lens.clone(),
                ))),
            }
        }
        changed
    }

    fn update_sizes(&mut self, sizes: [(f64, f64); N]) {
        for child in self.children.iter_mut() {
            child.widget_mut().update_sizes(sizes)
        }
        self.sizes = sizes
    }
}

impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone, const N: usize> Widget<R>
    for TreeNodeRootWidget<R, T, L, N>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        // match event {
        //     Event::MouseMove(_) => (),
        //     _ => eprintln!("{:?} {:?}", ctx.widget_id(), event),
        // }
        let event = match event {
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
                None
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
            _ => Some(event),
        };

        // get the unhandled event or return
        let event = if let Some(evt) = event {
            evt
        } else {
            return;
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
        let width = bc.max().width;

        let mut current_height = 0.;
        let mut paint_rect = Rect::ZERO;

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_bc = BoxConstraints::new(
                Size::new(width, basic_size),
                Size::new(width, f64::INFINITY),
            );
            let child_data = data.get_child(idx);
            let child_size = child.layout(ctx, &child_bc, child_data, env);
            let child_pos = Point::new(0., current_height);
            child.set_origin(ctx, child_data, env, child_pos);
            paint_rect = paint_rect.union(child.paint_rect());
            current_height += child_size.height;
        }

        let my_size = Size::new(width, current_height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        for (index, child_widget_node) in self.children.iter_mut().enumerate() {
            let child_tree_node = data.get_child(index);
            child_widget_node.paint(ctx, child_tree_node, env);
        }
    }
}

/// Tree Implementation
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize>
    Tree<R, T, L, N>
{
    pub fn new(
        header_widgets: [impl Widget<R> + 'static; N],
        make_widgets: [Arc<dyn Fn() -> Box<dyn Widget<T>>>; N],
        expand_lens: L,
    ) -> Self {
        let el = expand_lens.clone();
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(move || Box::new(make_wedge(el.clone()))));
        let header = Header::columns(header_widgets).draggable(true);
        let sizes = header.widget_pos();
        Tree {
            header: WidgetPod::new(header),
            root_node: WidgetPod::new(TreeNodeRootWidget::new(
                make_widgets,
                make_opener,
                sizes,
                expand_lens,
            )),
        }
    }

    // pub fn with_opener<W: Widget<T> + 'static>(
    //     mut self,
    //     closure: impl Fn() -> W + 'static,
    // ) -> Self {
    //     self.root_node.widget_mut().make_opener = Arc::new(Box::new(move || Box::new(closure())));
    //     self.root_node.widget_mut().opener = WidgetPod::new(Opener {
    //         widget: WidgetPod::new(self.root_node.widget_mut().make_opener.clone()()),
    //     });
    //     self
    // }
}

// Implement the Widget trait for Tree
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize> Widget<R>
    for Tree<R, T, L, N>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        match event {
            Event::Notification(notif) if notif.is(HEADER_SIZE_CHANGED) => {
                ctx.set_handled();
                let sizes = self.header.widget().widget_pos();
                self.root_node.widget_mut().update_sizes(sizes);
                ctx.request_layout();
                return;
            }
            _ => (),
        }
        self.header.event(ctx, event, data, env);
        self.root_node.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &R, env: &Env) {
        self.header.lifecycle(ctx, event, data, env);
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &R, data: &R, env: &Env) {
        self.header.update(ctx, data, env);
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &R, env: &Env) -> Size {
        let header_size = self.header.layout(ctx, bc, data, env);
        let node_bc = BoxConstraints::new(
            Size::new(header_size.width, 0.),
            Size::new(header_size.width, f64::INFINITY),
        );
        let content_size = self.root_node.layout(ctx, &node_bc, data, env);
        self.header.set_origin(ctx, data, env, Point::ORIGIN);
        self.root_node
            .set_origin(ctx, data, env, Point::new(0., header_size.height));
        // TODO: ctx.set_paint_insets...
        let my_size = Size::new(header_size.width, header_size.height + content_size.height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        self.header.paint(ctx, data, env);
        self.root_node.paint(ctx, data, env);
    }
}
