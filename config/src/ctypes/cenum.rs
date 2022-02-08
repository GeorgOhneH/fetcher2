use crate::ctypes::cstruct::CStruct;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::CType;
use crate::errors::Error;
use druid::im::{OrdMap, Vector};
use druid::widget::Label;
use druid::widget::{Flex, ListIter, Maybe};
use druid::Point;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx,
};
use druid::{Data, Lens};
use druid::{Widget, WidgetExt, WidgetPod};

use crate::widgets::warning_label::WarningLabel;
use crate::widgets::ListSelect;

#[derive(Debug, Clone, Data, Lens)]
pub struct CEnum {
    inner: Vector<CArg>,
    index_map: OrdMap<&'static str, usize>,
    name_map: OrdMap<usize, &'static str>,
    selected: Option<usize>,
    name: Option<&'static str>,
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

    pub fn get_selected(&self) -> Result<&CArg, Error> {
        self.selected
            .map(|idx| &self.inner[idx])
            .ok_or(Error::ValueRequired)
    }

    pub fn get_selected_mut(&mut self) -> Result<&mut CArg, Error> {
        self.selected
            .map(move |idx| &mut self.inner[idx])
            .ok_or(Error::ValueRequired)
    }

    pub fn set_selected(&mut self, variant: &str) -> Result<&CArg, Error> {
        match self.index_map.get(variant) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&self.inner[*i])
            }
            None => Err(Error::KeyDoesNotExist),
        }
    }

    pub fn set_selected_mut(&mut self, idx: &str) -> Result<&mut CArg, Error> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&mut self.inner[*i])
            }
            None => Err(Error::KeyDoesNotExist),
        }
    }

    pub fn is_leaf(&self) -> bool {
        todo!()
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
                Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}: ")))
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

    pub fn arg(&mut self, carg: CArg) {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(carg.name, idx);
        self.inner.name_map.insert(idx, carg.name);
        self.inner.inner.push_back(carg);
    }

    pub fn build(self) -> CEnum {
        self.inner
    }
}

#[derive(Debug, Clone, Data)]
pub enum CArgVariant {
    Unit,
    NewType(CType),
    Tuple(CTuple),
    Struct(CStruct),
}

impl CArgVariant {
    pub fn as_unit(&self) -> Result<(), Error> {
        match self {
            CArgVariant::Unit => Ok(()),
            _ => Err(Error::ExpectedUnitVariant),
        }
    }

    pub fn as_new_type(&self) -> Result<&CType, Error> {
        match self {
            CArgVariant::NewType(ty) => Ok(ty),
            _ => Err(Error::ExpectedNewTypeVariant),
        }
    }

    pub fn as_new_type_mut(&mut self) -> Result<&mut CType, Error> {
        match self {
            CArgVariant::NewType(ty) => Ok(ty),
            _ => Err(Error::ExpectedNewTypeVariant),
        }
    }

    pub fn as_tuple(&self) -> Result<&CTuple, Error> {
        match self {
            CArgVariant::Tuple(ctuple) => Ok(ctuple),
            _ => Err(Error::ExpectedTupleVariant),
        }
    }

    pub fn as_tuple_mut(&mut self) -> Result<&mut CTuple, Error> {
        match self {
            CArgVariant::Tuple(ctuple) => Ok(ctuple),
            _ => Err(Error::ExpectedTupleVariant),
        }
    }

    pub fn as_struct(&self) -> Result<&CStruct, Error> {
        match self {
            CArgVariant::Struct(cstruct) => Ok(cstruct),
            _ => Err(Error::ExpectedStructVariant),
        }
    }

    pub fn as_struct_mut(&mut self) -> Result<&mut CStruct, Error> {
        match self {
            CArgVariant::Struct(cstruct) => Ok(cstruct),
            _ => Err(Error::ExpectedStructVariant),
        }
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CArg {
    #[data(ignore)]
    #[lens(name = "name_lens")]
    pub name: &'static str,

    pub variant: CArgVariant,
}

impl CArg {
    pub fn new(name: &'static str, variant: CArgVariant) -> Self {
        Self { name, variant }
    }

    fn error_msg(&self) -> Option<String> {
        todo!()
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
