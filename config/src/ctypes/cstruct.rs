use std::sync::Arc;
use druid::im::Vector;
use druid::widget::{Container, CrossAxisAlignment, Flex, Label, List, ListIter, Maybe};
use druid::{im, Color};
use druid::{Data, Lens, Widget, WidgetExt};
use crate::ctypes::CType;
use crate::errors::Error;

use crate::widgets::warning_label::WarningLabel;

#[derive(Debug, Clone, Data, Lens)]
pub struct CStruct {
    pub inner: Vector<CKwarg>,
    #[data(ignore)]
    pub index_map: im::OrdMap<&'static str, usize>,
    #[data(ignore)]
    name: Option<&'static str>,
}

impl CStruct {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: im::OrdMap::new(),
            name: None,
        }
    }

    pub fn get(&self, name: &str) -> Result<&CKwarg, Error> {
        let idx = self.index_map.get(name).ok_or(Error::KeyDoesNotExist)?;
        Ok(&self.inner[*idx])
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut CKwarg, Error> {
        let idx = self.index_map.get(name).ok_or(Error::KeyDoesNotExist)?;
        Ok(&mut self.inner[*idx])
    }

    pub fn get_idx_ty_mut(&mut self, idx: usize) -> Option<&mut CType> {
        self.inner.get_mut(idx).map(|kwarg| &mut kwarg.ty)
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|str: &&'static str, _| str.to_string()))
                    .lens(Self::name),
            )
            .with_child(
                Container::new(List::new(CKwarg::widget).with_spacing(5.).padding(5.))
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
    pub fn arg(&mut self, arg: CKwarg) {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(&arg.name, idx);
        self.inner.inner.push_back(arg);
    }

    pub fn build(self) -> CStruct {
        self.inner
    }
}


#[derive(Debug, Clone, Data, Lens)]
pub struct CKwarg {
    #[data(ignore)]
    #[lens(name = "name_lens")]
    pub name: &'static str,
    #[data(ignore)]
    pub hint_text: Option<String>,
    pub ty: CType,
}

impl CKwarg {
    pub fn new(name: &'static str, ty: CType) -> Self {
        Self {
            ty,
            name,
            hint_text: None,
        }
    }


    fn error_msg(&self) -> Option<String> {
        todo!()
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(CType::widget().lens(Self::ty))
            .with_child(WarningLabel::new(|data: &Self| data.error_msg()))
    }
}
