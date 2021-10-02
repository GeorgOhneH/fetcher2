use druid::im::Vector;
use druid::widget::{Button, Controller, Flex, ListIter};
use druid::{im, Env, Event, EventCtx, Selector, Widget};
use druid::{Data, Lens, WidgetExt};

use crate::{CType, State};

#[derive(Debug, Clone, Data, Lens)]
pub struct CVec {
    inner: Vector<CItem>,
    #[data(ignore)]
    template_fn: fn() -> CType,
    #[data(ignore)]
    name: Option<String>,
}

impl ListIter<CItem> for CVec {
    fn for_each(&self, mut cb: impl FnMut(&CItem, usize)) {
        for (i, item) in self.inner.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut CItem, usize)) {
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

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(
                druid::widget::List::new(|| {
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
            .with_child(Button::new("Add").on_click(|ctx, c_vec: &mut Self, _env| {
                c_vec.push(c_vec.get_template());
                ctx.request_update();
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
                ctx.request_update();
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
