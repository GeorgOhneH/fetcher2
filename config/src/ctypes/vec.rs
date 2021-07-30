use crate::{CType, ConfigError, State};

use druid::im::Vector;
use druid::widget::{Button, Controller, Flex, ListIter};
use druid::{im, Env, Event, EventCtx, Selector, Widget, SingleUse, LifeCycleCtx, LifeCycle, UpdateCtx};
use druid::{lens, Data, Lens, LensExt, WidgetExt};
use serde_yaml::Sequence;
use std::ops::Deref;
use crate::widgets::List;

#[derive(Debug, Clone, Data, Lens)]
pub struct CVec {
    inner: Vector<CItem>,
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
    pub fn get_mut(&mut self) -> &mut im::Vector<CItem> {
        &mut self.inner
    }

    pub fn push(&mut self, ty: CType) {
        self.inner.push_back(CItem::new(ty, self.inner.len()))
    }

    pub fn remove(&mut self, idx: usize) {
        self.inner.remove(idx);
        for (i, item) in self.inner.iter_mut().enumerate() {
            item.index = i;
        }
    }

    pub fn get_template(&self) -> CType {
        (self.template_fn)()
    }

    pub fn set(&mut self, vec: im::Vector<CItem>) {
        self.inner = vec;
    }

    pub fn state(&self) -> State {
        self.inner.iter().map(|item| item.ty.state()).collect()
    }

    pub(crate) fn consume_sequence(&mut self, seq: Sequence) -> Result<(), ConfigError> {
        self.inner.clear();
        let mut result = Ok(());
        for value in seq {
            let mut template = self.get_template();
            match template.consume_value(value) {
                Ok(()) => self.push(template),
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
                            |ctx, item: &mut CItem, _env| {
                                ctx.submit_notification(DELETE.with(item.index))
                            },
                        ))
                })
                .controller(DeleteController::new()),
            )
            .with_child(Button::new("Add").on_click(|_, c_vec: &mut Self, _env| {
                c_vec.push(c_vec.get_template())
            }))
    }
}

pub const DELETE: Selector<usize> = Selector::new("fetcher2_config.vec.delete");

struct DeleteController {}

impl DeleteController {
    pub fn new() -> Self {
        Self {}
    }
}

impl<W: Widget<CVec>> Controller<CVec, W> for DeleteController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut CVec,
        env: &Env,
    ) {
        if let Event::Notification(notfi) = event {
            if let Some(idx) = notfi.get(DELETE) {
                data.remove(*idx);
                ctx.children_changed();
                ctx.set_handled()
            }
        };
        child.event(ctx, event, data, env)
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

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CVec {
        self.inner
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CItem {
    pub index: usize,
    pub ty: CType,
}

impl CItem {
    pub fn new(ty: CType, idx: usize) -> Self {
        Self { index: idx, ty }
    }
}

