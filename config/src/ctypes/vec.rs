use crate::{CType, ConfigError, State};

use druid::im::Vector;
use druid::widget::{Button, Flex, List, ListIter};
use druid::{im, Widget};
use druid::{lens, Data, Lens, LensExt, WidgetExt};
use serde_yaml::Sequence;

#[derive(Debug, Clone, Data, Lens)]
pub struct CVec {
    inner: im::Vector<CItem>,
    #[data(ignore)]
    template_fn: fn() -> CType,
    #[data(ignore)]
    name: Option<String>,
}

impl CVec {
    fn new(template_fn: fn() -> CType) -> Self {
        Self {
            inner: im::Vector::new(),
            template_fn,
            name: None,
        }
    }

    pub fn get(&self) -> &im::Vector<CItem> {
        &self.inner
    }

    pub fn get_template(&self) -> CType {
        (self.template_fn)()
    }

    pub fn set(&mut self, vec: im::Vector<CItem>) {
        self.inner = vec;
    }

    pub fn state(&self) -> State {
        self.inner
            .iter()
            .filter_map(|citem| {
                if citem.valid {
                    Some(citem.ty.state())
                } else {
                    None
                }
            })
            .collect()
    }

    pub(crate) fn consume_sequence(&mut self, seq: Sequence) -> Result<(), ConfigError> {
        self.inner.clear();
        let mut result = Ok(());
        for value in seq {
            let mut template = self.get_template();
            match template.consume_value(value) {
                Ok(()) => self.inner.push_back(CItem::new(template)),
                Err(err) => result = Err(err),
            }
        }
        result
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(
                List::new(|| {
                    Flex::row()
                        .with_child(CType::widget().lens(CItem::ty))
                        .with_child(Button::new("Delete").on_click(
                            |_ctx, item: &mut CItem, _env| {
                                // We have access to both child's data and shared data.
                                // Remove element from right list.
                                item.valid = false;
                            },
                        ))
                })
                .lens(CVec::inner.map(
                    |inner: &im::Vector<CItem>| inner.clone(),
                    |inner: &mut im::Vector<CItem>, mut data: im::Vector<CItem>| {
                        data.retain(|item| item.valid);
                        *inner = data;
                    },
                )),
            )
            .with_child(Button::new("Add").on_click(|_, c_vec: &mut Self, _env| {
                c_vec.inner.push_back(CItem::new(c_vec.get_template()))
            }))
    }
}

pub struct CVecBuilder {
    inner: CVec,
}

impl CVecBuilder {
    pub fn new(template: fn() -> CType) -> Self {
        Self {
            inner: CVec::new(template),
        }
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CVec {
        self.inner
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CItem {
    pub valid: bool,
    pub ty: CType,
}

impl CItem {
    pub fn new(ty: CType) -> Self {
        Self { valid: true, ty }
    }
}

impl From<CType> for CItem {
    fn from(ty: CType) -> Self {
        Self::new(ty)
    }
}
