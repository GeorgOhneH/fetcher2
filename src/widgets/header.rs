use druid::{Color, Cursor, Data, Point, Rect, theme, WidgetPod};
use druid::debug_state::DebugState;
use druid::kurbo::Line;
use druid::widget::{Axis, Label};
use druid::widget::prelude::*;

#[derive(Clone)]
pub struct HeaderConstrains<const N: usize> {
    pub sizes: [(f64, f64); N],
    pub max_width: f64,
}

impl<const N: usize> HeaderConstrains<N> {
    pub fn empty() -> Self {
        HeaderConstrains {
            sizes: [(0., 0.); N],
            max_width: 0.,
        }
    }
}

/// A container containing two other widgets, splitting the area either horizontally or vertically.
pub struct Header<T, const N: usize> {
    split_axis: Axis,
    wanted_size: [f64; N],
    effective_size: [f64; N],
    min_sizes: [f64; N],
    // Integers only
    bar_size: f64,
    // Integers only
    min_bar_area: f64,
    // Integers only
    solid: bool,
    draggable: bool,
    /// The split bar is hovered by the mouse. This state is locked to `true` if the
    /// widget is active (the bar is being dragged) to avoid cursor and painting jitter
    /// if the mouse moves faster than the layout and temporarily gets outside of the
    /// bar area while still being dragged.
    is_bar_hover: bool,
    /// Offset from the split point (bar center) to the actual mouse position when the
    /// bar was clicked. This is used to ensure a click without mouse move is a no-op,
    /// instead of re-centering the bar on the mouse.
    click_offset: f64,
    active_bar: Option<usize>,
    children: [WidgetPod<T, Box<dyn Widget<T>>>; N],
}

impl<T, const N: usize> Header<T, N> {
    /// Create a new split panel, with the specified axis being split in two.
    ///
    /// Horizontal split axis means that the children are left and right.
    /// Vertical split axis means that the children are up and down.
    fn new(split_axis: Axis, children: [impl Widget<T> + 'static; N]) -> Self {
        Header {
            split_axis,
            wanted_size: [100.0; N],
            effective_size: [0.; N],
            min_sizes: [0.0; N],
            bar_size: 6.0,
            min_bar_area: 6.0,
            solid: false,
            draggable: false,
            is_bar_hover: false,
            click_offset: 0.0,
            active_bar: None,
            children: children.map(|child| WidgetPod::new(child).boxed()),
        }
    }

    /// Create a new split panel, with the horizontal axis split in two by a vertical bar.
    /// The children are laid out left and right.
    pub fn columns(children: [impl Widget<T> + 'static; N]) -> Self {
        Self::new(Axis::Horizontal, children)
    }

    /// Create a new split panel, with the vertical axis split in two by a horizontal bar.
    /// The children are laid out up and down.
    pub fn rows(children: [impl Widget<T> + 'static; N]) -> Self {
        Self::new(Axis::Vertical, children)
    }

    /// Builder-style method to set the split point as a fraction of the split axis.
    ///
    /// The value must be between `0.0` and `1.0`, inclusive.
    /// The default split point is `0.5`.
    pub fn sizes(&mut self, size: [f64; N]) {
        self.wanted_size = size;
    }

    /// Builder-style method to set the minimum size for both sides of the split axis.
    ///
    /// The value must be greater than or equal to `0.0`.
    /// The value will be rounded up to the nearest integer.
    pub fn min_sizes(mut self, size: [f64; N]) -> Self {
        self.min_sizes = size;
        self
    }

    /// Builder-style method to set the size of the splitter bar.
    ///
    /// The value must be positive or zero.
    /// The value will be rounded up to the nearest integer.
    /// The default splitter bar size is `6.0`.
    pub fn bar_size(mut self, bar_size: f64) -> Self {
        assert!(bar_size >= 0.0, "bar_size must be 0.0 or greater!");
        self.bar_size = bar_size.ceil();
        self
    }

    /// Builder-style method to set the minimum size of the splitter bar area.
    ///
    /// The minimum splitter bar area defines the minimum size of the area
    /// where mouse hit detection is done for the splitter bar.
    /// The final area is either this or the splitter bar size, whichever is greater.
    ///
    /// This can be useful when you want to use a very narrow visual splitter bar,
    /// but don't want to sacrifice user experience by making it hard to click on.
    ///
    /// The value must be positive or zero.
    /// The value will be rounded up to the nearest integer.
    /// The default minimum splitter bar area is `6.0`.
    pub fn min_bar_area(mut self, min_bar_area: f64) -> Self {
        assert!(min_bar_area >= 0.0, "min_bar_area must be 0.0 or greater!");
        self.min_bar_area = min_bar_area.ceil();
        self
    }

    /// Builder-style method to set whether the split point can be changed by dragging.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Builder-style method to set whether the splitter bar is drawn as a solid rectangle.
    ///
    /// If this is `false` (the default), the bar will be drawn as two parallel lines.
    pub fn solid_bar(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }

    /// Returns the size of the splitter bar area.
    #[inline]
    fn bar_area(&self) -> f64 {
        self.bar_size.max(self.min_bar_area)
    }

    /// Returns the padding size added to each side of the splitter bar.
    #[inline]
    fn bar_padding(&self) -> f64 {
        (self.bar_area() - self.bar_size) / 2.0
    }

    fn widget_start(&self, idx: usize) -> f64 {
        let bar_area = self.bar_area();
        let mut total_size = 0.;
        for size in self.effective_size.iter().take(idx) {
            total_size += size + bar_area
        }
        total_size
    }

    fn widget_end(&self, idx: usize) -> f64 {
        self.widget_start(idx) + self.effective_size[idx]
    }

    pub fn constrains(&self) -> HeaderConstrains<N> {
        let bar_area = self.bar_area();
        let mut sizes = [(0., 0.); N];
        let mut total_size = 0.;
        for (idx, size) in self.effective_size.iter().enumerate() {
            sizes[idx] = (total_size, total_size + size);
            total_size += size + bar_area
        }
        HeaderConstrains {
            sizes,
            max_width: total_size,
        }
    }

    /// Returns the location of the edges of the splitter bar area,
    /// given the specified total size.
    fn bar_edges(&self) -> [(f64, f64); N] {
        let bar_area = self.bar_area();
        let mut result = [(0., 0.); N];
        let mut total_size = 0.;
        for (idx, size) in self.effective_size.iter().enumerate() {
            result[idx] = (size + total_size, total_size + size + bar_area);
            total_size += size + bar_area
        }
        result
    }

    /// Returns true if the provided mouse position is inside the splitter bar area.
    fn bar_hit_test(&self, size: Size, mouse_pos: Point) -> Option<usize> {
        let (m_pos, max_size) = match self.split_axis {
            Axis::Horizontal => (mouse_pos.x, size.width),
            Axis::Vertical => (mouse_pos.x, size.height),
        };
        for (idx, (edge1, edge2)) in self.bar_edges().iter().enumerate() {
            if max_size < *edge2 {
                break;
            }
            if m_pos >= *edge1 && m_pos <= *edge2 {
                return Some(idx);
            }
        }
        None
    }

    /// Set a new chosen split point.
    fn update_split_point(&mut self, idx: usize, mouse_pos: f64) {
        let min_limit = self.min_sizes[idx];
        let size = mouse_pos - self.widget_start(idx);
        self.wanted_size[idx] = size.max(min_limit);
    }

    /// Returns the color of the splitter bar.
    fn bar_color(&self, env: &Env) -> Color {
        if self.draggable {
            env.get(theme::BORDER_LIGHT)
        } else {
            env.get(theme::BORDER_DARK)
        }
    }

    fn paint_solid_bar(&mut self, ctx: &mut PaintCtx, env: &Env) {
        let size = ctx.size();
        let padding = self.bar_padding();
        for (edge1, edge2) in self.bar_edges() {
            let rect = match self.split_axis {
                Axis::Horizontal => Rect::from_points(
                    Point::new(edge1 + padding.ceil(), 0.0),
                    Point::new(edge2 - padding.floor(), size.height),
                ),
                Axis::Vertical => Rect::from_points(
                    Point::new(0.0, edge1 + padding.ceil()),
                    Point::new(size.width, edge2 - padding.floor()),
                ),
            };
            let splitter_color = self.bar_color(env);
            ctx.fill(rect, &splitter_color);
        }
    }
}

impl<T: Data, const N: usize> Widget<T> for Header<T, N> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        for child in self.children.iter_mut() {
            child.event(ctx, event, data, env);
        }

        if ctx.is_handled() {
            return;
        }

        if self.draggable {
            match event {
                Event::MouseDown(mouse) => {
                    if mouse.button.is_left() {
                        if let Some(idx) = self.bar_hit_test(ctx.size(), mouse.pos) {
                            ctx.set_handled();
                            ctx.set_active(true);
                            // Save the delta between the mouse click position and the split point
                            self.click_offset = match self.split_axis {
                                Axis::Horizontal => mouse.pos.x,
                                Axis::Vertical => mouse.pos.y,
                            } - self.widget_end(idx);
                            self.active_bar = Some(idx);
                            // If not already hovering, force and change cursor appropriately
                            if !self.is_bar_hover {
                                self.is_bar_hover = true;
                                match self.split_axis {
                                    Axis::Horizontal => ctx.set_cursor(&Cursor::ResizeLeftRight),
                                    Axis::Vertical => ctx.set_cursor(&Cursor::ResizeUpDown),
                                };
                            }
                        }
                    }
                }
                Event::MouseUp(mouse) => {
                    if mouse.button.is_left() && ctx.is_active() {
                        ctx.set_handled();
                        ctx.set_active(false);
                        // Dependending on where the mouse cursor is when the button is released,
                        // the cursor might or might not need to be changed
                        self.is_bar_hover =
                            ctx.is_hot() && self.bar_hit_test(ctx.size(), mouse.pos).is_some();
                        if !self.is_bar_hover {
                            ctx.clear_cursor()
                        }
                    }
                }
                Event::MouseMove(mouse) => {
                    if ctx.is_active() {
                        ctx.set_handled();
                        // If active, assume always hover/hot
                        let effective_pos = match self.split_axis {
                            Axis::Horizontal => mouse.pos.x,
                            Axis::Vertical => mouse.pos.y,
                        } - self.click_offset;
                        self.update_split_point(self.active_bar.unwrap(), effective_pos);
                        ctx.request_layout();
                    } else {
                        // If not active, set cursor when hovering state changes
                        let hover =
                            ctx.is_hot() && self.bar_hit_test(ctx.size(), mouse.pos).is_some();
                        if hover != self.is_bar_hover {
                            self.is_bar_hover = hover;
                            if hover {
                                match self.split_axis {
                                    Axis::Horizontal => ctx.set_cursor(&Cursor::ResizeLeftRight),
                                    Axis::Vertical => ctx.set_cursor(&Cursor::ResizeUpDown),
                                };
                            } else {
                                ctx.clear_cursor();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for child in self.children.iter_mut() {
            if !child.is_active() {
                child.event(ctx, event, data, env);
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        for child in self.children.iter_mut() {
            child.update(ctx, data, env);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        bc.debug_check("Split");
        let bar_area = self.bar_area();
        let mut my_size = Size::new(0., 0.);
        let mut current_length = 0.;
        let mut paint_rect = Rect::ZERO;
        let min_size = match self.split_axis {
            Axis::Horizontal => bc.min().width,
            Axis::Vertical => bc.min().height,
        };

        if N > 0 {
            let last_index = N - 1;
            let mut total_size = 0.;
            for (i, w_size) in self.wanted_size[..last_index].iter().enumerate() {
                let size = w_size.max(self.min_sizes[i]);
                self.effective_size[i] = size;
                total_size += size + bar_area;
            }
            let last_min_size = (min_size - total_size - bar_area).max(self.min_sizes[last_index]);
            self.effective_size[last_index] = last_min_size.max(self.wanted_size[last_index]);
        }

        for (idx, child) in self.children.iter_mut().enumerate() {
            let child_bc = match self.split_axis {
                Axis::Horizontal => {
                    let child_width = self.effective_size[idx];
                    BoxConstraints::new(
                        Size::new(child_width, 0.),
                        Size::new(child_width, f64::INFINITY),
                    )
                }
                Axis::Vertical => {
                    let child_height = self.effective_size[idx];
                    BoxConstraints::new(
                        Size::new(0., child_height),
                        Size::new(f64::INFINITY, child_height),
                    )
                }
            };
            let child_size = child.layout(ctx, &child_bc, data, env);
            let child_pos = match self.split_axis {
                Axis::Horizontal => {
                    my_size.height = my_size.height.max(child_size.height);
                    let p = Point::new(current_length, 0.0);
                    current_length += child_size.width + bar_area;
                    p
                }
                Axis::Vertical => {
                    my_size.width = my_size.width.max(child_size.width);
                    let p = Point::new(0.0, current_length);
                    current_length += child_size.height + bar_area;
                    p
                }
            };
            child.set_origin(ctx, data, env, child_pos);

            paint_rect = paint_rect.union(child.paint_rect());
        }

        // TODO?
        // let paint_rect = self.child1.paint_rect().union(self.child2.paint_rect());
        // let insets = paint_rect - my_size.to_rect();
        // ctx.set_paint_insets(insets);
        let insets = paint_rect - my_size.to_rect();
        ctx.set_paint_insets(insets);

        match self.split_axis {
            Axis::Horizontal => my_size.width = current_length,
            Axis::Vertical => my_size.height = current_length,
        }
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.paint_solid_bar(ctx, env);
        for child in self.children.iter_mut() {
            ctx.save().unwrap();
            ctx.clip(child.layout_rect());
            child.paint(ctx, data, env);
            ctx.restore().unwrap();
        }
    }
}
