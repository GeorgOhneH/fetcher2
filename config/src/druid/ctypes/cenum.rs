use druid::widget::{prelude::*, Flex, Label, ListIter, Maybe};
use druid::{Point, Widget, WidgetExt, WidgetPod};

use crate::ctypes::cenum::{CArg, CEnum};
use crate::druid::widgets::warning_label::WarningLabel;
use crate::druid::widgets::ListSelect;

impl CEnum {
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
                Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}: ")))
                    .lens(Self::name),
            )
            .with_child(list_select);
        Flex::column().with_child(x).with_child(CEnumWidget::new())
    }
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

impl CArg {
    fn error_msg(&self) -> Option<String> {
        Some("TODO".to_string())
    }
    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(Label::new("TODO"))
            .with_child(WarningLabel::new(|data: &Self| data.error_msg()))
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
        } else if let Some(idx) = data.selected {
            let widget = &mut self.widgets[idx];
            let mut new_child = data.inner[idx].to_owned();
            widget.event(ctx, event, &mut new_child, env);
            if !new_child.same(&data.inner[idx]) {
                data.inner[idx] = new_child
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
        } else if let Some(idx) = data.selected {
            let widget = &mut self.widgets[idx];
            widget.lifecycle(ctx, event, &data.inner[idx], env)
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
