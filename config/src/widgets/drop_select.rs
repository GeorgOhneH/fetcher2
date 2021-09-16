// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A simple list selection widget, for selecting a single value out of a list.

use std::fmt::Debug;
use std::marker::PhantomData;

use druid::{
    Affine, BoxConstraints, Data, Env, Event, EventCtx, Insets, LayoutCtx, Lens, LensExt, LifeCycle,
    LifeCycleCtx, LinearGradient, PaintCtx, Point, RenderContext, Size, theme, UnitPoint,
    UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use druid::commands::CLOSE_WINDOW;
use druid::kurbo::BezPath;
use druid::widget::{
    Controller, DefaultScopePolicy, Label, LabelText, LineBreaking, ListIter, Scope,
};
use druid_widget_nursery::{AutoFocus, Dropdown, Wedge, WidgetExt as _};
use druid_widget_nursery::dropdown::{DROPDOWN_CLOSED, DROPDOWN_HIDE, DROPDOWN_SHOW};

use crate::widgets::ListSelect;

// NOTE: This is copied from Button. Should those be generic, or maybe set in the environment?
const LABEL_INSETS: Insets = Insets::uniform_xy(8., 2.);

/// Builds a list selection widget, showed as a button, for which the different possible values appear as a dropdown.
pub struct DropdownSelect<P, T, L> {
    _p: PhantomData<P>,
    _t: PhantomData<T>,
    _l: PhantomData<L>,
}

impl<P, T, L> DropdownSelect<P, T, L>
where
    P: ListIter<T> + Debug,
    T: Data,
    L: Lens<P, Option<usize>> + Clone + 'static,
{
    /// Given a vector of `(label_text, enum_variant)` tuples, create a dropdown select widget
    /// This is exactly the same interface as `Radio` so that both can be used interchangably,
    /// with dropdown taking less space in the UI.
    pub fn new<W: Widget<T> + 'static>(
        list_closure: impl Fn(&P, usize) -> W + Clone + 'static,
        selected_lens: L,
    ) -> impl Widget<P> {
        Self::new_inner(list_closure, selected_lens, None)
    }

    pub fn new_sized<W: Widget<T> + 'static>(
        list_closure: impl Fn(&P, usize) -> W + Clone + 'static,
        selected_lens: L,
        size: Size,
    ) -> impl Widget<P> {
        Self::new_inner(list_closure, selected_lens, Some(size))
    }

    fn new_inner<W: Widget<T> + 'static>(
        list_closure: impl Fn(&P, usize) -> W + Clone + 'static,
        selected_lens: L,
        size: Option<Size>,
    ) -> impl Widget<P> {
        let header = DropdownButton::new(move |_t: &P, _env: &Env| "test".to_string())
            .on_click(|ctx: &mut EventCtx, p: &mut DropdownState<P>, _| {
                if p.expanded {
                    p.expanded = false;
                    ctx.submit_notification(DROPDOWN_HIDE)
                } else {
                    p.expanded = true;
                    ctx.submit_notification(DROPDOWN_SHOW)
                }
            })
            .on_command(DROPDOWN_CLOSED, |_ctx, &(), p: &mut DropdownState<P>| {
                p.expanded = false;
            });

        let make_drop = move |_p: &DropdownState<P>, env: &Env| {
            let w = ListSelect::new(list_closure.clone(), selected_lens.clone())
                .lens(DropdownState::<P>::data)
                .border(env.get(theme::BORDER_DARK), 1.0)
                .controller(DropdownSelectCtrl::new(selected_lens.clone()))
                .controller(AutoFocus);
            if let Some(size) = size {
                w.fix_size(size.width, size.height).boxed()
            } else {
                w.boxed()
            }
        };
        // A `Scope` is used here to add internal data shared within the children widgets,
        // namely whether or not the dropdown is expanded. See `DropdownState`.
        Scope::new(
            DefaultScopePolicy::from_lens(DropdownState::new, druid::lens!(DropdownState<P>, data)),
            Dropdown::new(header, make_drop),
        )
    }
}

// This controller will send itself "COLLAPSE" events whenever the dropdown is removed, and
// reacts to it by updating its expanded state
struct DropdownSelectCtrl<P, T, L> {
    selected_lens: L,
    _p: PhantomData<P>,
    _t: PhantomData<T>,
}

impl<P, T, L> DropdownSelectCtrl<P, T, L>
where
    P: ListIter<T> + Debug,
    T: Data,
    L: Lens<P, Option<usize>> + Clone + 'static,
{
    pub fn new(selected_lens: L) -> Self {
        Self {
            selected_lens,
            _p: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<P, T, L, W> Controller<DropdownState<P>, W> for DropdownSelectCtrl<P, T, L>
where
    P: ListIter<T> + Debug,
    T: Data,
    L: Lens<P, Option<usize>> + Clone + 'static,
    W: Widget<DropdownState<P>>,
{
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &DropdownState<P>,
        data: &DropdownState<P>,
        env: &Env,
    ) {
        if self.selected_lens.get(&old_data.data) != self.selected_lens.get(&data.data) {
            // workaround for https://github.com/linebender/druid/issues/1939
            let ext = ctx.get_external_handle();
            ext.submit_command(CLOSE_WINDOW, (), ctx.window_id())
                .unwrap();
        }
        child.update(ctx, old_data, data, env);
    }
}

#[derive(Clone, Data, Lens, Debug)]
struct DropdownState<T> {
    data: T,
    expanded: bool,
}

impl<T> DropdownState<T> {
    fn new(data: T) -> Self {
        DropdownState {
            data,
            expanded: false,
        }
    }
}

/// A button with a left or down arrow, changing shape when opened.
struct DropdownButton<T> {
    wedge: WidgetPod<bool, Wedge>,
    label: Label<T>,
    label_size: Size,
}

impl<T: Data> DropdownButton<T> {
    fn new(text: impl Into<LabelText<T>>) -> DropdownButton<T> {
        DropdownButton::from_label(Label::new(text).with_line_break_mode(LineBreaking::Clip))
    }

    fn from_label(label: Label<T>) -> DropdownButton<T> {
        DropdownButton {
            wedge: WidgetPod::new(Wedge::new()),
            label,
            label_size: Size::ZERO,
        }
    }
}

impl<T: Data> Widget<DropdownState<T>> for DropdownButton<T> {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        _data: &mut DropdownState<T>,
        _env: &Env,
    ) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &DropdownState<T>,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
        self.wedge.lifecycle(ctx, event, &data.expanded, env);
        self.label.lifecycle(ctx, event, &data.data, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &DropdownState<T>,
        data: &DropdownState<T>,
        env: &Env,
    ) {
        if old_data.expanded != data.expanded {
            ctx.request_paint();
        }
        self.wedge.update(ctx, &data.expanded, env);
        self.label.update(ctx, &old_data.data, &data.data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &DropdownState<T>,
        env: &Env,
    ) -> Size {
        let padding = Size::new(LABEL_INSETS.x_value(), LABEL_INSETS.y_value());
        let label_bc = bc.shrink(padding).loosen();
        self.label_size = self.label.layout(ctx, &label_bc, &data.data, env);
        // HACK: to make sure we look okay at default sizes when beside a textbox,
        // we make sure we will have at least the same height as the default textbox.
        let min_height = env.get(theme::BORDERED_WIDGET_HEIGHT);
        let height = (self.label_size.height + padding.height).max(min_height);
        let baseline = self.label.baseline_offset();
        ctx.set_baseline_offset(baseline + LABEL_INSETS.y1);

        let basic_width = env.get(theme::BASIC_WIDGET_HEIGHT);
        let wedge_bc = BoxConstraints::tight(Size::new(basic_width, basic_width));
        let wedge_pos = Point::new(0.0, (height - basic_width) / 2.0);
        self.wedge.layout(ctx, &wedge_bc, &data.expanded, env);
        self.wedge.set_origin(ctx, &data.expanded, env, wedge_pos);

        bc.constrain(Size::new(
            self.label_size.width + padding.width + basic_width,
            height,
        ))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DropdownState<T>, env: &Env) {
        let is_active = ctx.is_active();
        let is_hot = ctx.is_hot();
        let size = ctx.size();
        let stroke_width = env.get(theme::BUTTON_BORDER_WIDTH);
        let basic_width = env.get(theme::BASIC_WIDGET_HEIGHT);

        let bg_gradient = if is_active {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::BUTTON_DARK), env.get(theme::BUTTON_LIGHT)),
            )
        } else {
            LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::BUTTON_LIGHT), env.get(theme::BUTTON_DARK)),
            )
        };

        let border_color = if is_hot {
            env.get(theme::BORDER_LIGHT)
        } else {
            env.get(theme::BORDER_DARK)
        };

        let radius = env.get(theme::BUTTON_BORDER_RADIUS) * 1.5;
        if data.expanded {
            let rounded_rect =
                half_rounded_rect(size - Size::new(stroke_width, stroke_width), radius);
            ctx.with_save(|ctx| {
                ctx.transform(Affine::translate((stroke_width / 2.0, stroke_width / 2.0)));
                ctx.fill(rounded_rect.clone(), &bg_gradient);
                ctx.stroke(rounded_rect.clone(), &border_color, stroke_width);
            });
        } else {
            let rounded_rect = size
                .to_rect()
                .inset(-stroke_width / 2.0)
                .to_rounded_rect(radius);
            ctx.fill(rounded_rect, &bg_gradient);
            ctx.stroke(rounded_rect, &border_color, stroke_width);
        }

        let label_offset_y = (size.height - self.label_size.height) / 2.0;

        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((basic_width, label_offset_y)));
            self.label.paint(ctx, &data.data, env);
        });

        self.wedge.paint(ctx, &data.expanded, env);
    }
}

// This returns a shape approximating a rectangle with only the top corners rounded
fn half_rounded_rect(size: Size, r: f64) -> BezPath {
    let radius = r.min(size.width / 2.0).min(size.height / 2.0);
    let quad_r = radius * (1.0 - 4.0 * (2.0_f64.sqrt() - 1.0) / 3.0); // see https://stackoverflow.com/a/27863181
    let mut path = BezPath::new();
    path.move_to((radius, 0.0));
    path.line_to((size.width - radius, 0.0));
    path.curve_to(
        (size.width - quad_r, 0.0),
        (size.width, quad_r),
        (size.width, radius),
    );
    path.line_to((size.width, size.height));
    path.line_to((0.0, size.height));
    path.line_to((0.0, radius));
    path.curve_to((0.0, quad_r), (quad_r, 0.0), (radius, 0.0));
    path.close_path();
    path
}
