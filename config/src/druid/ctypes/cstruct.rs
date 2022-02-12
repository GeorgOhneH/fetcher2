use druid::widget::{Container, CrossAxisAlignment, Flex, Label, List, ListIter, Maybe};
use druid::Color;
use druid::{Data, Widget, WidgetExt};

use crate::ctypes::cstruct::{CKwarg, CStruct};
use crate::ctypes::CType;
use crate::druid::widgets::warning_label::WarningLabel;

impl CStruct {
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

impl CKwarg {
    pub fn error_msg(&self) -> Option<String> {
        todo!()
    }
    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(CType::widget().lens(Self::ty))
            .with_child(WarningLabel::new(|data: &Self| data.error_msg()))
    }
}
