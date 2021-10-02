use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx,
};
use druid::{Data, Lens};
use druid::{Widget, WidgetExt, WidgetPod};
use druid::im::{OrdMap, Vector};
use druid::lens::Index;
use druid::LensExt;
use druid::Point;
use druid::widget::{Flex, ListIter, Maybe};
use druid::widget::Label;

use crate::{CType, InvalidError, State};
use crate::widgets::ListSelect;
use crate::widgets::warning_label::WarningLabel;

#[derive(Debug, Clone, Data, Lens)]
pub struct CEnum {
    inner: Vector<CArg>,
    index_map: OrdMap<String, usize>,
    name_map: OrdMap<usize, String>,
    selected: Option<usize>,
    name: Option<String>,
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

impl CEnum {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: OrdMap::new(),
            name_map: OrdMap::new(),
            selected: None,
            name: None,
        }
    }

    pub fn get_selected(&self) -> Option<&CArg> {
        self.selected.map(|idx| &self.inner[idx])
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut CArg> {
        self.selected.map(move |idx| &mut self.inner[idx])
    }

    pub fn unselect(&mut self) {
        self.selected = None
    }

    pub fn set_selected(&mut self, idx: &str) -> Result<&CArg, InvalidError> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&self.inner[*i])
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn set_selected_mut(&mut self, idx: &str) -> Result<&mut CArg, InvalidError> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&mut self.inner[*i])
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn state(&self) -> State {
        match self.get_selected() {
            Some(carg) => carg.state(),
            None => State::None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self.get_selected() {
            None => true,
            Some(carg) => carg.is_unit(),
        }
    }

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
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ": "))
                    .lens(Self::name),
            )
            .with_child(list_select);
        Flex::column().with_child(x).with_child(CEnumWidget::new())
    }
}

pub struct CEnumBuilder {
    inner: CEnum,
}

impl CEnumBuilder {
    pub fn new() -> Self {
        Self {
            inner: CEnum::new(),
        }
    }

    pub fn arg(mut self, carg: CArg) -> Self {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(carg.name().clone(), idx);
        self.inner.name_map.insert(idx, carg.name().clone());
        self.inner.inner.push_back(carg);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn default<T: Into<String>>(mut self, name: T) -> Self {
        self.inner
            .set_selected(&name.into())
            .expect("Default for EnumConfig doesn't exist");
        self
    }

    pub fn build(self) -> CEnum {
        self.inner
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct Parameter {
    required: bool,
    ty: CType,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CArg {
    #[data(ignore)]
    #[lens(name = "name_lens")]
    name: String,

    parameter: Option<Parameter>,
}

impl CArg {
    fn new(name: String) -> Self {
        Self {
            name,
            parameter: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn get(&self) -> Option<&CType> {
        self.parameter.as_ref().map(|par| &par.ty)
    }

    pub fn get_mut(&mut self) -> Option<&mut CType> {
        self.parameter.as_mut().map(|par| &mut par.ty)
    }

    pub fn is_unit(&self) -> bool {
        self.parameter.is_none()
    }

    pub fn state(&self) -> State {
        match &self.parameter {
            Some(parameter) => match parameter.ty.state() {
                State::Valid => State::Valid,
                State::None => {
                    if parameter.required {
                        State::invalid("Value is required")
                    } else {
                        State::Valid
                    }
                }
                State::InValid(msg) => State::InValid(msg),
            },
            None => State::Valid,
        }
    }

    fn error_msg(&self) -> Option<String> {
        match self.state() {
            State::InValid(str) => Some(str),
            _ => None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_child(Maybe::or_empty(|| CType::widget().lens(Parameter::ty)).lens(Self::parameter))
            .with_child(WarningLabel::new(|data: &Self| data.error_msg()))
    }
}

pub struct CArgBuilder {
    inner: CArg,
}

impl CArgBuilder {
    pub fn new(name: String) -> Self {
        Self {
            inner: CArg::new(name),
        }
    }

    pub fn value(mut self, ty: CType, required: bool) -> Self {
        self.inner.parameter = Some(Parameter { ty, required });
        self
    }

    pub fn build(self) -> CArg {
        self.inner
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
        } else {
            if let Some(idx) = data.selected {
                let widget = &mut self.widgets[idx];
                let mut new_child = data.inner[idx].to_owned();
                widget.event(ctx, event, &mut new_child, env);
                if !new_child.same(&data.inner[idx]) {
                    data.inner[idx] = new_child
                }
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
        } else {
            if let Some(idx) = data.selected {
                let widget = &mut self.widgets[idx];
                widget.lifecycle(ctx, event, &data.inner[idx], env)
            }
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
