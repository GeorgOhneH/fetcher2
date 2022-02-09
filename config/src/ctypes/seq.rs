use druid::im::Vector;
use druid::widget::{Button, Controller, Flex, ListIter};
use druid::{im, Env, Event, EventCtx, Selector, Widget};
use druid::{Data, Lens, WidgetExt};
use crate::ctypes::CType;

#[derive(Debug, Clone, Data, Lens)]
pub struct CSeq {
    pub inner: Vector<CItem>,
    #[data(ignore)]
    pub template: Box<CType>,
    #[data(ignore)]
    name: Option<&'static str>,
}

impl CSeq {
    pub fn new(template: CType) -> Self {
        Self {
            inner: im::Vector::new(),
            template: Box::new(template),
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

    pub fn set(&mut self, vec: im::Vector<CItem>) {
        self.inner = vec;
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
                c_vec.push(c_vec.template.as_ref().clone());
                ctx.request_update();
            }))
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

impl ListIter<CItem> for CSeq {
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

pub const DELETE: Selector<usize> = Selector::new("fetcher2_config.vec.delete");

struct DeleteController {}

impl DeleteController {
    pub fn new() -> Self {
        Self {}
    }
}


impl<W: Widget<CSeq>> Controller<CSeq, W> for DeleteController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut CSeq,
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
