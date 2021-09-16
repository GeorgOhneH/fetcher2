use druid::widget::{
    Container, CrossAxisAlignment, Flex, FlexParams, Label, List, ListIter, MainAxisAlignment,
    Maybe,
};
use druid::{im, Color};
use druid::{Data, Lens, Widget, WidgetExt};
use std::collections::hash_map::Iter;
use std::fmt;

use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, WidgetPod,
};

use crate::{CType, ConfigError, RequiredError, State};
use druid::im::Vector;
use druid::widget::SizedBox;
use druid_widget_nursery::WidgetExt as _;
use serde::de::Error;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

#[derive(Debug, Clone, Data, Lens)]
pub struct CStruct {
    pub inner: Vector<CKwarg>,
    #[data(ignore)]
    index_map: im::OrdMap<String, usize>,
    #[data(ignore)]
    name: Option<String>,
}

impl CStruct {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: im::OrdMap::new(),
            name: None,
        }
    }

    pub fn get(&self, name: &str) -> Option<&CKwarg> {
        let idx = self.index_map.get(name)?;
        Some(self.inner.get(*idx).unwrap())
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut CKwarg> {
        let idx = self.index_map.get(name)?;
        Some(self.inner.get_mut(*idx).unwrap())
    }

    pub fn get_ty(&self, name: &str) -> Option<&CType> {
        match self.get(name) {
            Some(config_arg) => Some(config_arg.get()),
            None => None,
        }
    }

    pub fn get_ty_mut(&mut self, name: &str) -> Option<&mut CType> {
        match self.get_mut(name) {
            Some(config_arg) => Some(config_arg.get_mut()),
            None => None,
        }
    }

    pub fn get_idx_ty_mut(&mut self, idx: usize) -> Option<&mut CType> {
        self.inner.get_mut(idx).map(|kwarg| &mut kwarg.ty)
    }

    pub fn iter(&self) -> im::vector::Iter<CKwarg> {
        self.inner.iter()
    }

    pub fn state(&self) -> State {
        self.inner.iter().map(|ckwarg| ckwarg.state()).collect()
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone()))
                    .lens(Self::name),
            )
            .with_child(
                Container::new(List::new(|| CKwarg::widget()).with_spacing(5.).padding(5.))
                    .border(Color::GRAY, 2.),
            )
    }
}

impl ListIter<CKwarg> for CStruct {
    fn for_each(&self, cb: impl FnMut(&CKwarg, usize)) {
        self.inner.for_each(cb)
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut CKwarg, usize)) {
        for (index, element) in self.inner.clone().iter().enumerate() {
            let mut new_element = element.to_owned();
            cb(&mut new_element, index);
            if !new_element.same(element) {
                self.inner[index] = new_element;
            }
        }
    }

    fn data_len(&self) -> usize {
        self.inner.data_len()
    }
}

pub struct CStructBuilder {
    inner: CStruct,
}

impl CStructBuilder {
    pub fn new() -> Self {
        Self {
            inner: CStruct::new(),
        }
    }
    pub fn arg(mut self, arg: CKwarg) -> Self {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(arg.name().clone(), idx);
        self.inner.inner.push_back(arg);
        self
    }

    pub fn name<T: Into<String>>(mut self, name: T) -> Self {
        self.inner.name = Some(name.into());
        self
    }

    pub fn build(self) -> CStruct {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub enum InactiveBehavior {
    GrayOut,
    Hide,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CKwarg {
    #[data(ignore)]
    required: bool,
    #[data(ignore)]
    #[lens(name = "name_lens")]
    name: String,
    #[data(ignore)]
    hint_text: Option<String>,
    #[data(ignore)]
    active_fn: fn(CStruct) -> bool,
    #[data(ignore)]
    inactive_behavior: InactiveBehavior,
    ty: CType,
}

impl CKwarg {
    fn new(name: String, ty: CType) -> Self {
        Self {
            ty,
            name,
            active_fn: |_app| true,
            hint_text: None,
            inactive_behavior: InactiveBehavior::GrayOut,
            required: true,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn get(&self) -> &CType {
        &self.ty
    }

    pub fn get_mut(&mut self) -> &mut CType {
        &mut self.ty
    }

    pub fn state(&self) -> State {
        match self.ty.state() {
            State::Valid => State::Valid,
            State::None => {
                if self.required {
                    State::invalid("Value is required")
                } else {
                    State::Valid
                }
            }
            State::InValid(msg) => State::InValid(msg),
        }
    }

    fn error_msg(&self) -> Option<String> {
        if !self.ty.is_leaf() {
            None
        } else {
            match self.ty.state() {
                State::Valid => None,
                State::None => {
                    if self.required {
                        Some("Value is required".to_string())
                    } else {
                        None
                    }
                }
                State::InValid(msg) => Some(msg),
            }
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(CType::widget().lens(Self::ty))
            .with_child(WarningLabel::new())
    }
}

pub struct CKwargBuilder {
    inner: CKwarg,
}

impl CKwargBuilder {
    pub fn new(name: String, ty: CType) -> Self {
        Self {
            inner: CKwarg::new(name, ty),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.inner.required = required;
        self
    }

    pub fn hint_text(mut self, hint_text: String) -> Self {
        self.inner.hint_text = Some(hint_text);
        self
    }

    pub fn inactive_behavior(mut self, behavior: InactiveBehavior) -> Self {
        self.inner.inactive_behavior = behavior;
        self
    }

    pub fn active_fn(mut self, active_fn: fn(CStruct) -> bool) -> Self {
        self.inner.active_fn = active_fn;
        self
    }

    pub fn build(self) -> CKwarg {
        self.inner
    }
}

pub struct WarningLabel {
    label: Label<()>,
    active: bool,
}

impl WarningLabel {
    pub fn new() -> WarningLabel {
        WarningLabel {
            label: Label::new("test").with_text_color(Color::rgb8(255, 0, 0)),
            active: true,
        }
    }
}

impl Widget<CKwarg> for WarningLabel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut CKwarg, env: &Env) {
        self.label.event(ctx, event, &mut (), env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CKwarg, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            match data.error_msg() {
                None => self.active = false,
                Some(msg) => {
                    self.active = true;
                    self.label.set_text(msg)
                }
            }
            ctx.request_layout();
        }
        self.label.lifecycle(ctx, event, &(), env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CKwarg, data: &CKwarg, env: &Env) {
        if !old_data.same(data) {
            match data.error_msg() {
                None => self.active = false,
                Some(msg) => {
                    self.active = true;
                    self.label.set_text(msg);
                }
            }
            ctx.request_layout();
        }
        self.label.update(ctx, &(), &(), env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &CKwarg,
        env: &Env,
    ) -> Size {
        if self.active {
            self.label.layout(ctx, bc, &(), env)
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &CKwarg, env: &Env) {
        if self.active {
            self.label.paint(ctx, &(), env);
        }
    }
}
