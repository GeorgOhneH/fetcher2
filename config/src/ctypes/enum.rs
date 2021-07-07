use crate::*;
use druid::widget::Flex;
use druid::{im, Widget, WidgetExt, WidgetPod};
use druid::{Data, Lens};
use serde_yaml::{Mapping, Value};

use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};

#[derive(Debug, Clone, Data)]
pub struct CEnum {
    inner: im::OrdMap<String, CArg>,
    selected: Option<String>,
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: im::OrdMap::new(),
            selected: None,
        }
    }

    pub fn get_selected(&self) -> Option<&CArg> {
        self.selected
            .as_ref()
            .map(|idx| self.inner.get(idx).unwrap())
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut CArg> {
        match &self.selected {
            Some(idx) => Some(self.inner.get_mut(idx).unwrap()),
            None => None,
        }
    }

    pub fn unselect(&mut self) {
        self.selected = None
    }

    pub fn set_selected(&mut self, idx: String) -> Result<&CArg, InvalidError> {
        match self.inner.get(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn set_selected_mut(&mut self, idx: String) -> Result<&mut CArg, InvalidError> {
        match self.inner.get_mut(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        if map.len() != 1 {
            Err(InvalidError::new("Enum map has the wrong format").into())
        } else if let Some((vkey, value)) = map.into_iter().next() {
            let key = match vkey {
                Value::String(str) => str,
                _ => return Err(InvalidError::new("map key is not String").into()),
            };
            let carg = self.set_selected_mut(key)?;
            carg.consume_value(value)
        } else {
            panic!("Should never happen")
        }
    }

    pub fn widget() -> impl Widget<Self> {
        ListSelect::new()
    }
}

pub struct CEnumBuilder {
    inner: CEnum,
}

impl CEnumBuilder {
    pub fn new() -> Self {
        Self {
            inner: CEnum::new(),
        }
    }

    pub fn arg(mut self, carg: CArg) -> Self {
        self.inner.inner.insert(carg.name.clone(), carg);
        self
    }

    pub fn build(self) -> CEnum {
        self.inner
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CArg {
    #[data(ignore)]
    #[lens(name = "name_lens")]
    name: String,
    #[data(ignore)]
    gui_name: Option<String>,
    parameter: Option<CType>,
}

impl CArg {
    fn new(name: String) -> Self {
        Self {
            name,
            gui_name: None,
            parameter: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn get(&self) -> Option<&CType> {
        Option::from(&self.parameter)
    }

    pub fn get_mut(&mut self) -> Option<&mut CType> {
        Option::from(&mut self.parameter)
    }

    pub fn is_unit(&self) -> bool {
        self.parameter.is_none()
    }

    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        match &mut self.parameter {
            Some(ctype) => ctype.consume_value(value),
            None => {
                if let Value::String(_) = value {
                    Ok(())
                } else {
                    Err(InvalidError::new("Unit Enum must be a String").into())
                }
            }
        }
    }
}

pub struct CArgBuilder {
    inner: CArg,
}

impl CArgBuilder {
    pub fn new(name: String) -> Self {
        Self {
            inner: CArg::new(name),
        }
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.gui_name = Some(name);
        self
    }

    pub fn value(mut self, value: CType) -> Self {
        self.inner.parameter = Some(value);
        self
    }

    pub fn build(self) -> CArg {
        self.inner
    }

}
use druid::widget::{Click, Container, ControllerHost, DefaultScopePolicy, Scope};
use druid::{
    theme, BoxConstraints, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    LinearGradient, PaintCtx, RenderContext, Selector, Size, UnitPoint, UpdateCtx,
};
use druid_widget_nursery::{Dropdown, Wedge, DROP};
use std::marker::PhantomData;

use druid::keyboard_types::Key;
use druid::widget::{Controller, CrossAxisAlignment, Label, LabelText};
//
// // NOTE: This is copied from Button. Should those be generic, or maybe set in the environment?
// const LABEL_INSETS: Insets = Insets::uniform_xy(8., 2.);
// const COLLAPSE: Selector<()> = Selector::new("druid-widget-nursery.dropdown.collapse");
// //
// // /// Builds a list selection widget, showed as a button, for which the different possible values appear as a dropdown.
// pub struct DropdownSelect;
// //
// impl DropdownSelect {
//     /// Given a vector of `(label_text, enum_variant)` tuples, create a dropdown select widget
//     /// This is exactly the same interface as `Radio` so that both can be used interchangably,
//     /// with dropdown taking less space in the UI.
//     pub fn new(
//         values: impl IntoIterator<Item = String> + Clone + 'static,
//     ) -> impl Widget<Option<String>> {
//         Self::new_inner(values, None)
//     }
//
//     fn new_inner(
//         values: impl IntoIterator<Item = String> + Clone + 'static,
//         size: Option<Size>,
//     ) -> impl Widget<Option<String>> {
//         let header = DropdownButton::new(|t: &String, _: &Env| t)
//             .on_click(|ctx: &mut EventCtx, t: &mut DropdownState, _| {
//                 if t.expanded {
//                     t.expanded = false;
//                     ctx.submit_command(COLLAPSE.to(ctx.widget_id()));
//                 } else {
//                     t.expanded = true;
//                     ctx.submit_command(DROP.to(ctx.widget_id()))
//                 }
//             });
//
//         let make_drop = move |_t: &DropdownState, env: &Env| {
//             ControllerHost::new(
//                 ListSelect::new(values.clone())
//                     .lens(DropdownState::data)
//                     .border(env.get(theme::BORDER_DARK), 1.0),
//                 DropdownSelectController { _t: PhantomData },
//             )
//         };
//         // A `Scope` is used here to add internal data shared within the children widgets,
//         // namely whether or not the dropdown is expanded. See `DropdownState`.
//         Scope::new(
//             DefaultScopePolicy::from_lens(DropdownState::new, druid::lens!(DropdownState, data)),
//             if let Some(size) = size {
//                 Dropdown::new_sized(header, make_drop, size)
//             } else {
//                 Dropdown::new(header, make_drop)
//             },
//         )
//     }
// }
//
// // This controller will send itself "COLLAPSE" events whenever the dropdown is removed, and
// // reacts to it by updating its expanded state
// struct DropdownSelectController<T> {
//     _t: PhantomData<T>,
// }
//
// impl<T: Data + PartialEq> Controller<DropdownState, Container<DropdownState>>
// for DropdownSelectController<T>
// {
//     fn event(
//         &mut self,
//         child: &mut Container<DropdownState>,
//         ctx: &mut EventCtx,
//         event: &Event,
//         data: &mut DropdownState,
//         env: &Env,
//     ) {
//         match event {
//             Event::Command(n) if n.is(COLLAPSE) => {
//                 data.expanded = false;
//             }
//             _ => child.event(ctx, event, data, env),
//         }
//     }
//     fn lifecycle(
//         &mut self,
//         child: &mut Container<DropdownState>,
//         ctx: &mut LifeCycleCtx,
//         event: &LifeCycle,
//         data: &DropdownState,
//         env: &Env,
//     ) {
//         if let LifeCycle::HotChanged(false) = event {
//             ctx.submit_command(COLLAPSE);
//         }
//         child.lifecycle(ctx, event, data, env)
//     }
//
//     fn update(
//         &mut self,
//         child: &mut Container<DropdownState>,
//         ctx: &mut UpdateCtx,
//         old_data: &DropdownState,
//         data: &DropdownState,
//         env: &Env,
//     ) {
//         ctx.submit_command(COLLAPSE);
//         child.update(ctx, old_data, data, env)
//     }
// }
//
// #[derive(Clone, Data, Lens)]
// struct DropdownState {
//     data: String,
//     expanded: bool,
// }
//
// impl DropdownState {
//     fn new(data: String) -> Self {
//         DropdownState {
//             data,
//             expanded: false,
//         }
//     }
// }
//
// /// A button with a left or down arrow, changing shape when opened.
// struct DropdownButton {
//     wedge: WidgetPod<bool, Wedge>,
//     label: Label<String>,
//     label_size: Size,
// }
//
// impl DropdownButton {
//     fn new(text: impl Into<LabelText<String>>) -> DropdownButton {
//         DropdownButton::from_label(Label::new(text))
//     }
//
//     fn from_label(label: Label<String>) -> DropdownButton {
//         DropdownButton {
//             wedge: WidgetPod::new(Wedge::new()),
//             label,
//             label_size: Size::ZERO,
//         }
//     }
//
//     fn on_click(
//         self,
//         f: impl Fn(&mut EventCtx, &mut DropdownState, &Env) + 'static,
//     ) -> ControllerHost<Self, Click<DropdownState>> {
//         ControllerHost::new(self, Click::new(f))
//     }
// }
//
// impl Widget<DropdownState> for DropdownButton {
//     fn event(
//         &mut self,
//         ctx: &mut EventCtx,
//         event: &Event,
//         _data: &mut DropdownState,
//         _env: &Env,
//     ) {
//         match event {
//             Event::MouseDown(_) => {
//                 ctx.set_active(true);
//                 ctx.request_paint();
//             }
//             Event::MouseUp(_) => {
//                 if ctx.is_active() {
//                     ctx.set_active(false);
//                     ctx.request_paint();
//                 }
//             }
//             _ => (),
//         }
//     }
//
//     fn lifecycle(
//         &mut self,
//         ctx: &mut LifeCycleCtx,
//         event: &LifeCycle,
//         data: &DropdownState,
//         env: &Env,
//     ) {
//         if let LifeCycle::HotChanged(_) = event {
//             ctx.request_paint();
//         }
//         self.wedge.lifecycle(ctx, event, &data.expanded, env);
//         self.label.lifecycle(ctx, event, &data.data, env)
//     }
//
//     fn update(
//         &mut self,
//         ctx: &mut UpdateCtx,
//         old_data: &DropdownState,
//         data: &DropdownState,
//         env: &Env,
//     ) {
//         if old_data.expanded != data.expanded {
//             ctx.request_paint();
//         }
//         self.wedge.update(ctx, &data.expanded, env);
//         self.label.update(ctx, &old_data.data, &data.data, env)
//     }
//
//     fn layout(
//         &mut self,
//         ctx: &mut LayoutCtx,
//         bc: &BoxConstraints,
//         data: &DropdownState,
//         env: &Env,
//     ) -> Size {
//         let padding = Size::new(LABEL_INSETS.x_value(), LABEL_INSETS.y_value());
//         let label_bc = bc.shrink(padding).loosen();
//         self.label_size = self.label.layout(ctx, &label_bc, &data.data, env);
//         // HACK: to make sure we look okay at default sizes when beside a textbox,
//         // we make sure we will have at least the same height as the default textbox.
//         let min_height = env.get(theme::BORDERED_WIDGET_HEIGHT);
//         let height = (self.label_size.height + padding.height).max(min_height);
//         let baseline = self.label.baseline_offset();
//         ctx.set_baseline_offset(baseline + LABEL_INSETS.y1);
//
//         let basic_width = env.get(theme::BASIC_WIDGET_HEIGHT);
//         let wedge_bc = BoxConstraints::tight(Size::new(basic_width, basic_width));
//         let wedge_pos = Point::new(0.0, (height - basic_width) / 2.0);
//         self.wedge.layout(ctx, &wedge_bc, &data.expanded, env);
//         self.wedge.set_origin(ctx, &data.expanded, env, wedge_pos);
//
//         bc.constrain(Size::new(
//             self.label_size.width + padding.width + basic_width,
//             height,
//         ))
//     }
//
//     fn paint(&mut self, ctx: &mut PaintCtx, data: &DropdownState, env: &Env) {
//         let is_active = ctx.is_active();
//         let is_hot = ctx.is_hot();
//         let size = ctx.size();
//         let stroke_width = env.get(theme::BUTTON_BORDER_WIDTH);
//         let basic_width = env.get(theme::BASIC_WIDGET_HEIGHT);
//
//         let bg_gradient = if is_active {
//             LinearGradient::new(
//                 UnitPoint::TOP,
//                 UnitPoint::BOTTOM,
//                 (env.get(theme::BUTTON_DARK), env.get(theme::BUTTON_LIGHT)),
//             )
//         } else {
//             LinearGradient::new(
//                 UnitPoint::TOP,
//                 UnitPoint::BOTTOM,
//                 (env.get(theme::BUTTON_LIGHT), env.get(theme::BUTTON_DARK)),
//             )
//         };
//
//         let border_color = if is_hot {
//             env.get(theme::BORDER_LIGHT)
//         } else {
//             env.get(theme::BORDER_DARK)
//         };
//
//         let radius = env.get(theme::BUTTON_BORDER_RADIUS) * 1.5;
//         if data.expanded {
//             let rounded_rect =
//                 half_rounded_rect(size - Size::new(stroke_width, stroke_width), radius);
//             ctx.with_save(|ctx| {
//                 ctx.transform(Affine::translate((stroke_width / 2.0, stroke_width / 2.0)));
//                 ctx.fill(rounded_rect.clone(), &bg_gradient);
//                 ctx.stroke(rounded_rect.clone(), &border_color, stroke_width);
//             });
//         } else {
//             let rounded_rect = size
//                 .to_rect()
//                 .inset(-stroke_width / 2.0)
//                 .to_rounded_rect(radius);
//             ctx.fill(rounded_rect, &bg_gradient);
//             ctx.stroke(rounded_rect, &border_color, stroke_width);
//         }
//
//         let label_offset_y = (size.height - self.label_size.height) / 2.0;
//
//         ctx.with_save(|ctx| {
//             ctx.transform(Affine::translate((basic_width, label_offset_y)));
//             self.label.paint(ctx, &data.data, env);
//         });
//
//         self.wedge.paint(ctx, &data.expanded, env);
//     }
// }
//
// // This returns a shape approximating a rectangle with only the top corners rounded
// fn half_rounded_rect(size: Size, r: f64) -> BezPath {
//     let radius = r.min(size.width / 2.0).min(size.height / 2.0);
//     let quad_r = radius * (1.0 - 4.0 * (2.0_f64.sqrt() - 1.0) / 3.0); // see https://stackoverflow.com/a/27863181
//     let mut path = BezPath::new();
//     path.move_to((radius, 0.0));
//     path.line_to((size.width - radius, 0.0));
//     path.curve_to(
//         (size.width - quad_r, 0.0),
//         (size.width, quad_r),
//         (size.width, radius),
//     );
//     path.line_to((size.width, size.height));
//     path.line_to((0.0, size.height));
//     path.line_to((0.0, radius));
//     path.curve_to((0.0, quad_r), (quad_r, 0.0), (radius, 0.0));
//     path.close_path();
//     path
// }

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

// added padding between the edges of the widget and the text.
const LABEL_X_PADDING: f64 = 8.0;

/// Builds a simple list selection widget, for selecting a single value out of a list.
pub struct ListSelect {
    /// Internal widget data.
    widget: Flex<CEnum>,
    /// A controller handling item selection.
    controller: ListSelectController,
}

impl ListSelect {
    /// Given a vector of `(label_text, enum_variant)` tuples, create a list of items to select from
    pub fn new() -> ListSelect {
        ListSelect {
            widget: Flex::row(),
            controller: ListSelectController { action: None },
        }
    }

    /// Provide a closure to be called when an item is selected.
    pub fn on_select(self, f: impl Fn(&mut EventCtx, &mut CEnum, &Env) + 'static) -> ListSelect {
        let widget = self.widget;

        ListSelect {
            widget,
            controller: ListSelectController {
                action: Some(Box::new(f)),
            },
        }
    }
}

impl Widget<CEnum> for ListSelect {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CEnum, env: &Env) {
        self.controller
            .event(&mut self.widget, ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CEnum, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let mut col = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);
            for label in data.inner.keys().into_iter() {
                col.add_child(ListItem::new(label.clone()));
            }
            self.widget = col;
            ctx.request_paint();
            ctx.request_layout();
        }
        self.controller
            .lifecycle(&mut self.widget, ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CEnum, data: &CEnum, env: &Env) {
        self.controller
            .update(&mut self.widget, ctx, old_data, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CEnum,
        env: &Env,
    ) -> Size {
        self.widget.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CEnum, env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

type ListSelectAction = Box<dyn Fn(&mut EventCtx, &mut CEnum, &Env) + 'static>;

// A Controller to handle arrow key in the list selection widget.
struct ListSelectController {
    action: Option<ListSelectAction>,
}

impl ListSelectController {
    fn change_index(&self, data: &mut CEnum, next_else_previous: bool) {
        if let Some(mut index) = data
            .inner
            .keys()
            .position(|variant| Some(variant) == data.selected.as_ref())
        {
            if next_else_previous {
                index += 1
            } else if index > 0 {
                index -= 1
            }
            if let Some(new_data) = data.inner.keys().skip(index).next() {
                data.selected = Some(new_data.clone());
            }
        }
    }
}

impl Controller<CEnum, Flex<CEnum>> for ListSelectController {
    fn event(
        &mut self,
        child: &mut Flex<CEnum>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut CEnum,
        env: &Env,
    ) {
        let mut selected = false;

        if let Event::MouseDown(_) = event {
            selected = true;
            ctx.request_focus();
        }
        if let Event::KeyDown(key_event) = event {
            match key_event.key {
                Key::ArrowUp => {
                    selected = true;
                    self.change_index(data, false);
                    ctx.request_update();
                }
                Key::ArrowDown => {
                    selected = true;
                    self.change_index(data, true);
                    ctx.request_update();
                }
                _ => {}
            }
        } else {
            child.event(ctx, event, data, env)
        }

        // fire the callback if a valid index was selected
        if selected {
            if let Some(cb) = &self.action {
                cb(ctx, data, env);
            }
        }
    }
}

pub struct ListItem {
    label: String,
    child_label: Label<String>,
    label_y: f64,
}

impl ListItem {
    /// Create a single ListItem from label text and an enum variant
    pub fn new(label: String) -> Self {
        Self {
            label: label.clone(),
            child_label: Label::new(label),
            label_y: 0.0,
        }
    }
}

impl Widget<CEnum> for ListItem {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CEnum, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        data.selected = Some(self.label.clone());
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CEnum, env: &Env) {
        self.child_label.lifecycle(ctx, event, &self.label, env);
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CEnum, data: &CEnum, env: &Env) {
        self.child_label.update(ctx, &self.label, &self.label, env);
        if !old_data.same(data) {
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CEnum,
        env: &Env,
    ) -> Size {
        let label_size = self.child_label.layout(ctx, &bc.loosen(), &self.label, env);
        let height = (env.get(theme::BASIC_WIDGET_HEIGHT)
            + env.get(theme::WIDGET_PADDING_VERTICAL))
        .max(label_size.height);
        self.label_y = (height - label_size.height) / 2.0;
        bc.constrain(Size::new(label_size.width + LABEL_X_PADDING * 2.0, height))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CEnum, env: &Env) {
        let border_width = 1.0;
        let rect = ctx.size().to_rect().inset(-border_width / 2.0);

        // Paint the data in the primary color if we are selected
        if Some(&self.label) == data.selected.as_ref() {
            let background_gradient = LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::PRIMARY_LIGHT), env.get(theme::PRIMARY_DARK)),
            );
            ctx.fill(rect, &background_gradient);
        } else if ctx.is_active() {
            let background_gradient = LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::BACKGROUND_LIGHT),
                    env.get(theme::BACKGROUND_DARK),
                ),
            );
            ctx.fill(rect, &background_gradient);
        }

        // Paint a light rectangle around the item if hot
        if ctx.is_hot() {
            ctx.stroke(rect, &env.get(theme::BORDER_LIGHT), 1.);
        }

        // Paint the text label
        self.child_label
            .draw_at(ctx, (LABEL_X_PADDING, self.label_y));
    }
}
