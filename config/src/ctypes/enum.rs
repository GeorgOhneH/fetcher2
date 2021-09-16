use druid::widget::{Button, Either, Flex, ListIter, Maybe, ViewSwitcher};
use druid::{im, LensExt, Widget, WidgetExt, WidgetPod};
use druid::{Data, Lens};

use druid::kurbo::BezPath;
use druid::piet::{FontFamily, ImageFormat, InterpolationMode, Text, TextLayoutBuilder};
use druid::widget::prelude::*;
use druid::{
    Affine, AppLauncher, Color, FontDescriptor, LocalizedString, Point, Rect, TextLayout,
    WindowDesc,
};

use druid::widget::{Click, Container, ControllerHost, DefaultScopePolicy, Scope};
use druid::{
    theme, BoxConstraints, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    LinearGradient, PaintCtx, RenderContext, Selector, Size, UnitPoint, UpdateCtx,
};
use druid_widget_nursery::dropdown::{Dropdown, DROPDOWN_SHOW};
use druid_widget_nursery::Wedge;
use std::marker::PhantomData;

use crate::widgets::drop_select::DropdownSelect;
use crate::widgets::ListSelect;
use druid::im::{OrdMap, Vector};
use druid::keyboard_types::Key;
use druid::lens::Identity;
use druid::widget::{Controller, CrossAxisAlignment, Label, LabelText};
use crate::{State, InvalidError, CType};

#[derive(Debug, Clone, Data, Lens)]
pub struct CEnum {
    inner: Vector<CArg>,
    index_map: OrdMap<String, usize>,
    name_map: OrdMap<usize, String>,
    selected: Option<usize>,
    name: Option<String>,
}

impl ListIter<CArg> for CEnum {
    fn for_each(&self, mut cb: impl FnMut(&CArg, usize)) {
        for (i, item) in self.inner.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut CArg, usize)) {
        for (index, element) in self.inner.clone().iter().enumerate() {
            let mut new_element = element.to_owned();
            cb(&mut new_element, index);
            if !new_element.same(element) {
                self.inner[index] = new_element;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.inner.len()
    }
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: OrdMap::new(),
            name_map: OrdMap::new(),
            selected: None,
            name: None,
        }
    }

    pub fn get_selected(&self) -> Option<&CArg> {
        self.selected.map(|idx| &self.inner[idx])
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut CArg> {
        self.selected.map(move |idx| &mut self.inner[idx])
    }

    pub fn unselect(&mut self) {
        self.selected = None
    }

    pub fn set_selected(&mut self, idx: &str) -> Result<&CArg, InvalidError> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&self.inner[*i])
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn set_selected_mut(&mut self, idx: &str) -> Result<&mut CArg, InvalidError> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&mut self.inner[*i])
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn state(&self) -> State {
        match self.get_selected() {
            Some(carg) => carg.state(),
            None => State::None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self.get_selected() {
            None => true,
            Some(carg) => carg.is_unit(),
        }
    }

    pub fn widget() -> impl Widget<Self> {
        let list_select = ListSelect::new(
            |data: &Self, idx| {
                let name = data.name_map.get(&idx).unwrap();
                Label::new(name.clone())
            },
            Self::selected,
        )
        .horizontal();

        let x = Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ": "))
                    .lens(Self::name),
            )
            .with_child(list_select);
        Flex::column().with_child(x).with_child(CEnumWidget::new())
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
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(carg.name().clone(), idx);
        self.inner.name_map.insert(idx, carg.name().clone());
        self.inner.inner.push_back(carg);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn default(mut self, name: String) -> Self {
        self.inner
            .set_selected(&name)
            .expect("Default for EnumConfig doesn't exist");
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

struct CEnumWidget {
    widgets: Vec<WidgetPod<CArg, Box<dyn Widget<CArg>>>>,
}

impl CEnumWidget {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }
}

impl Widget<CEnum> for CEnumWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CEnum, env: &Env) {
        if event.should_propagate_to_hidden() {
            for (idx, widget) in self.widgets.iter_mut().enumerate() {
                let mut new_child = data.inner[idx].to_owned();
                widget.event(ctx, event, &mut new_child, env);
                if !new_child.same(&data.inner[idx]) {
                    data.inner[idx] = new_child
                }
            }
        } else {
            if let Some(idx) = data.selected {
                let widget = &mut self.widgets[idx];
                let mut new_child = data.inner[idx].to_owned();
                widget.event(ctx, event, &mut new_child, env);
                if !new_child.same(&data.inner[idx]) {
                    data.inner[idx] = new_child
                }
            }
        }
    }
    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CEnum, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.widgets = data
                .inner
                .iter()
                .map(|_| WidgetPod::new(CArg::widget().boxed()))
                .collect::<Vec<_>>();
            ctx.children_changed();
        }
        if event.should_propagate_to_hidden() {
            for (idx, widget) in self.widgets.iter_mut().enumerate() {
                widget.lifecycle(ctx, event, &data.inner[idx], env)
            }
        } else {
            if let Some(idx) = data.selected {
                let widget = &mut self.widgets[idx];
                widget.lifecycle(ctx, event, &data.inner[idx], env)
            }
        }
    }
    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CEnum, data: &CEnum, env: &Env) {
        if old_data.selected != data.selected {
            ctx.request_layout()
        }
        for (idx, widget) in self.widgets.iter_mut().enumerate() {
            widget.update(ctx, &data.inner[idx], env)
        }
    }
    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CEnum,
        env: &Env,
    ) -> Size {
        if let Some(idx) = data.selected {
            let widget = &mut self.widgets[idx];
            let size = widget.layout(ctx, bc, &data.inner[idx], env);
            widget.set_origin(ctx, &data.inner[idx], env, Point::ORIGIN);
            size
        } else {
            bc.min()
        }
    }
    fn paint(&mut self, ctx: &mut PaintCtx, data: &CEnum, env: &Env) {
        if let Some(idx) = data.selected {
            let widget = &mut self.widgets[idx];
            widget.paint(ctx, &data.inner[idx], env);
        }
    }
}
