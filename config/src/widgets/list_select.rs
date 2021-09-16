use std::cmp::Ordering;
use std::f64;
use std::marker::PhantomData;

use druid::{LensExt, RenderContext, theme, WidgetExt};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, UpdateCtx, Widget, widget::Axis, WidgetPod,
};
use druid::kurbo::{Point, Rect, Size};
use druid::widget::ListIter;

enum UpdateSource {
    Event(Option<usize>),
    Data,
}

pub struct ListSelect<P, T, L>
where
    P: ListIter<T>,
    T: Data,
    L: Lens<P, Option<usize>>,
{
    closure: Box<dyn Fn(&P, usize) -> Box<dyn Widget<T>>>,
    children: Vec<WidgetPod<T, ListItem<T>>>,
    selected_lens: L,
    axis: Axis,
    _marker: PhantomData<P>,
}

impl<P, T, L> ListSelect<P, T, L>
where
    P: ListIter<T>,
    T: Data,
    L: Lens<P, Option<usize>>,
{
    /// Create a new list widget. Closure will be called every time when a new child
    /// needs to be constructed.
    pub fn new<W: Widget<T> + 'static>(
        closure: impl Fn(&P, usize) -> W + 'static,
        selected_lens: L,
    ) -> Self {
        ListSelect {
            closure: Box::new(move |data, idx| Box::new(closure(data, idx))),
            children: Vec::new(),
            selected_lens,
            axis: Axis::Vertical,
            _marker: PhantomData,
        }
    }

    /// Sets the widget to display the list horizontally, not vertically.
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    /// When the widget is created or the data changes, create or remove children as needed
    ///
    /// Returns `true` if children were added or removed.
    fn update_child_count(&mut self, data: &P, _env: &Env) -> bool {
        let len = self.children.len();
        match len.cmp(&data.data_len()) {
            Ordering::Greater => self.children.truncate(data.data_len()),
            Ordering::Less => data.for_each(|_, i| {
                if i >= len {
                    let child = WidgetPod::new(ListItem::new((self.closure)(data, i)));
                    self.children.push(child);
                }
            }),
            Ordering::Equal => (),
        }
        len != data.data_len()
    }

    fn update_selection(&mut self, new_idx: Option<usize>) -> bool {
        let current = self.currently_selected();

        for child in self.children.iter_mut() {
            child.widget_mut().selected = false;
        }
        if let Some(idx) = new_idx {
            self.children[idx].widget_mut().selected = true;
        }
        new_idx != current
    }

    fn currently_selected(&self) -> Option<usize> {
        for (i, widget) in self.children.iter().enumerate() {
            if widget.widget().selected {
                return Some(i);
            }
        }
        None
    }

    fn at(&self, p: Point) -> Option<usize> {
        for (i, widget) in self.children.iter().enumerate() {
            if widget.layout_rect().contains(p) {
                return Some(i);
            }
        }
        None
    }
}

impl<P, T, L> Widget<P> for ListSelect<P, T, L>
where
    P: ListIter<T>,
    T: Data,
    L: Lens<P, Option<usize>>,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut P, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each_mut(|child_data, _| {
            if let Some(child) = children.next() {
                child.event(ctx, event, child_data, env);
            }
        });

        if ctx.is_handled() {
            return;
        }

        match event {
            Event::MouseDown(mouse) => {
                if !ctx.is_disabled() {
                    if let Some(idx) = self.at(mouse.pos) {
                        if self.update_selection(Some(idx)) {
                            ctx.request_paint();
                            self.selected_lens.put(data, Some(idx))
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &P, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_child_count(data, env) {
                ctx.children_changed();
            }
            if self.update_selection(self.selected_lens.get(data)) {
                ctx.request_paint();
            }
        }

        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.lifecycle(ctx, event, child_data, env);
            }
        });
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &P, data: &P, env: &Env) {
        // we send update to children first, before adding or removing children;
        // this way we avoid sending update to newly added children, at the cost
        // of potentially updating children that are going to be removed.
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.update(ctx, child_data, env);
            }
        });

        if self.update_child_count(data, env) {
            ctx.children_changed();
        }

        if self.currently_selected() != self.selected_lens.get(data) {
            if self.update_selection(self.selected_lens.get(data)) {
                ctx.request_paint();
            }
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &P, env: &Env) -> Size {
        let axis = self.axis;
        let mut minor = axis.minor(bc.min());
        let mut major_pos = 0.0;
        let mut paint_rect = Rect::ZERO;
        let mut children = self.children.iter_mut();
        let child_bc = match axis {
            Axis::Horizontal => BoxConstraints::new(
                Size::new(0., bc.min().height),
                Size::new(f64::INFINITY, bc.max().height),
            ),
            Axis::Vertical => BoxConstraints::new(
                Size::new(bc.min().width, 0.),
                Size::new(bc.max().width, f64::INFINITY),
            ),
        };
        data.for_each(|child_data, _| {
            let child = match children.next() {
                Some(child) => child,
                None => {
                    return;
                }
            };
            let child_size = child.layout(ctx, &child_bc, child_data, env);
            let child_pos: Point = axis.pack(major_pos, 0.).into();
            child.set_origin(ctx, child_data, env, child_pos);
            paint_rect = paint_rect.union(child.paint_rect());
            minor = minor.max(axis.minor(child_size));
            major_pos += axis.major(child_size);
        });

        let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor)));
        let insets = paint_rect - my_size.to_rect();
        ctx.set_paint_insets(insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &P, env: &Env) {
        let mut children = self.children.iter_mut();
        data.for_each(|child_data, _| {
            if let Some(child) = children.next() {
                child.paint(ctx, child_data, env);
            }
        });
    }
}

struct ListItem<T: Data> {
    pub selected: bool,
    child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> ListItem<T> {
    pub fn new(child: impl Widget<T> + 'static) -> Self {
        Self {
            child: WidgetPod::new(child.boxed()),
            selected: false,
        }
    }
}

impl<T: Data> Widget<T> for ListItem<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.child.event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::HotChanged(_) | LifeCycle::DisabledChanged(_) = event {
            ctx.request_paint()
        }
        self.child.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.child.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, data, env);
        self.child.set_origin(ctx, data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let rect = ctx.size().to_rect();
        if ctx.is_disabled() {
            if self.selected {
                ctx.fill(rect, &env.get(theme::DISABLED_FOREGROUND_DARK));
            } else {
                ctx.fill(rect, &env.get(theme::BACKGROUND_LIGHT));
            }
        } else {
            if self.selected {
                ctx.fill(rect, &env.get(theme::PRIMARY_DARK));
            } else if ctx.is_hot() {
                ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));
            } else {
                ctx.fill(rect, &env.get(theme::BACKGROUND_LIGHT));
            }
        }
        self.child.paint(ctx, data, env)
    }
}
