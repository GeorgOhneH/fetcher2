use druid::{Color, im};

use druid::{Data, Lens, Widget, WidgetExt};
use druid::im::Vector;
use druid::widget::{Container, CrossAxisAlignment, Flex, Label, List, ListIter, Maybe};

use crate::{CType, State};
use crate::widgets::warning_label::WarningLabel;

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

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(CType::widget().lens(Self::ty))
            .with_child(WarningLabel::new(|data: &Self| data.error_msg()))
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
