use crate::*;
use druid::widget::{Either, Flex, Maybe, ViewSwitcher};
use druid::{im, LensExt, Widget, WidgetExt, WidgetPod};
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
    inner: OrdMap<String, CArg>,
    selected: Option<String>,
    name: Option<String>,
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: im::OrdMap::new(),
            selected: None,
            name: None,
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

    pub fn state(&self) -> State {
        match self.get_selected() {
            Some(carg) => carg.state(),
            None => State::None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        // TODO
        true
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(ListSelect::new())
            .with_child(CEnumWidget::new())
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

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn default(mut self, name: String) -> Self {
        self.inner.set_selected(name).expect("Default for EnumConfig doesn't exist");
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
    parameter: Option<CType>,
}

impl CArg {
    fn new(name: String) -> Self {
        Self {
            name,
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

    pub fn state(&self) -> State {
        match &self.parameter {
            Some(ty) => ty.state(),
            None => State::Valid,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Maybe::or_empty(|| CType::widget()).lens(Self::parameter)
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
use druid_widget_nursery::{Dropdown, Wedge};
use std::marker::PhantomData;

use druid::keyboard_types::Key;
use druid::lens::Identity;
use druid::widget::{Controller, CrossAxisAlignment, Label, LabelText};
use druid::im::OrdMap;


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

pub struct ListSelect {
    widget: Flex<CEnum>,
}

impl ListSelect {
    pub fn new() -> ListSelect {
        ListSelect {
            widget: Flex::column(),
        }
    }
}

impl ListSelect {
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

impl Widget<CEnum> for ListSelect {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CEnum, env: &Env) {
        if let Event::MouseDown(_) = event {
            ctx.request_focus();
        }
        if let Event::KeyDown(key_event) = event {
            match key_event.key {
                Key::ArrowUp => {
                    self.change_index(data, false);
                    // ctx.request_update();
                }
                Key::ArrowDown => {
                    self.change_index(data, true);
                    // ctx.request_update();
                }
                _ => {}
            }
        } else {
            self.widget.event(ctx, event, data, env)
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CEnum, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            for label in data.inner.keys().into_iter() {
                self.widget.add_child(ListItem::new(label.clone()));
            }
            ctx.request_paint();
            ctx.request_layout();
            ctx.children_changed()
        }
        self.widget.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CEnum, data: &CEnum, env: &Env) {
        self.widget.update(ctx, old_data, data, env)
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

struct CEnumWidget {
    widgets: HashMap<String, WidgetPod<CArg, Box<dyn Widget<CArg>>>>,
}

impl CEnumWidget {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }
}

impl Widget<CEnum> for CEnumWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CEnum, env: &Env) {
        if event.should_propagate_to_hidden() {
            for (key, widget) in self.widgets.iter_mut() {
                widget.event(ctx, event, data.inner.get_mut(key).unwrap(), env)
            }
        } else {
            if let Some(data_name) = &data.selected {
                let widget = self.widgets.get_mut(data_name).unwrap();
                widget.event(ctx, event, data.inner.get_mut(data_name).unwrap(), env)
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CEnum, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            for key in data.inner.keys() {
                self.widgets
                    .insert(key.clone(), WidgetPod::new(CArg::widget().boxed()));
            }
            // ctx.children_changed();
        }
        if event.should_propagate_to_hidden() {
            for (key, widget) in self.widgets.iter_mut() {
                widget.lifecycle(ctx, event, data.inner.get(key).unwrap(), env);
            }
        } else {
            if let Some(data_name) = &data.selected {
                let widget = self.widgets.get_mut(data_name).unwrap();
                widget.lifecycle(ctx, event, data.inner.get(data_name).unwrap(), env);
            }
        }
    }
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CEnum, data: &CEnum, env: &Env) {
        if old_data.selected == data.selected {
            if let Some(data_name) = &data.selected {
                let widget = self.widgets.get_mut(data_name).unwrap();
                widget.update(ctx, data.inner.get(data_name).unwrap(), env);
            }
        } else {
            // ctx.request_paint();
            // ctx.children_changed();
            if let Some(data_name) = &data.selected {
                let widget = self.widgets.get_mut(data_name).unwrap();
                widget.update(ctx, data.inner.get(data_name).unwrap(), env);
            }
        }
    }
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CEnum,
        env: &Env,
    ) -> Size {
        if let Some(data_name) = &data.selected {
            let widget = self.widgets.get_mut(data_name).unwrap();
            let size = widget.layout(ctx, bc, data.inner.get(data_name).unwrap(), env);
            widget.set_layout_rect(ctx, data.inner.get(data_name).unwrap(), env, size.to_rect());
            size
        } else {
            bc.min()
        }
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &CEnum, env: &Env) {
        if let Some(data_name) = &data.selected {
            let widget = self.widgets.get_mut(data_name).unwrap();
            widget.paint(ctx, data.inner.get(data_name).unwrap(), env);
        }
    }
}
