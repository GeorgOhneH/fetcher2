use druid::im::Vector;
use druid::widget::{Button, Controller, Flex, ListIter};
use druid::{im, Env, Event, EventCtx, Selector, Widget};
use druid::{Data, Lens, WidgetExt};
use crate::ctypes::CType;
use crate::ctypes::seq::{CItem, CSeq};

impl CSeq {
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
