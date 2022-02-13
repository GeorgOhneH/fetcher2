use config::ctypes::cstruct::CStruct;
use config::ctypes::CType;
use config::deserializer::ConfigDeserializer;
use config::serializer::ConfigSerializer;
use config::traveller::{ConfigTraveller, Travel};
use druid::commands::CLOSE_WINDOW;
use druid::widget::{Button, Flex};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Size, Target, UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use druid_widget_nursery::selectors;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

selectors! {
    APPLY
}

type OnChangeFn<T> = Box<dyn Fn(&mut EventCtx, &Option<T>, &mut T, &Env)>;

pub struct CTypeBuffer<T> {
    pub child: WidgetPod<CType, Box<dyn Widget<CType>>>,
    pub ty: CType,
    pub on_change_fn: Option<OnChangeFn<T>>,
}

impl<T: Travel + Data> CTypeBuffer<T> {
    pub fn new(child: impl Widget<CType> + 'static) -> Self {
        let ty = T::travel(&mut ConfigTraveller::new()).expect("Travel struct in not valid");
        Self {
            child: WidgetPod::new(child.boxed()),
            ty,
            on_change_fn: None,
        }
    }

    pub fn on_change(&mut self, on_change_fn: OnChangeFn<T>) {
        self.on_change_fn = Some(on_change_fn)
    }

    pub fn with_name(&mut self, name: &'static str) {
        self.ty.set_name(name)
    }
}

impl<T: Travel + Data + DeserializeOwned + Serialize> Widget<Option<T>> for CTypeBuffer<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Option<T>, env: &Env) {
        match event {
            Event::Command(command) if command.is(APPLY) => {
                ctx.set_handled();
                if let Ok(mut new_data) = T::deserialize(&mut ConfigDeserializer::new(&self.ty)) {
                    if let Some(on_change_fn) = &self.on_change_fn {
                        (on_change_fn)(ctx, data, &mut new_data, env)
                    }
                    *data = Some(new_data);
                    ctx.submit_command(CLOSE_WINDOW);
                } else {
                    dbg!("INVALID DATA FOUND");
                }
            }
            _ => (),
        }

        let old_data = self.ty.clone();
        self.child.event(ctx, event, &mut self.ty, env);
        if !old_data.same(&self.ty) {
            dbg!("DATA CHANGED");
            ctx.request_update()
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Option<T>,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(init) = data {
                // TODO: not unwrap
                init.serialize(&mut ConfigSerializer::new(&mut self.ty))
                    .unwrap();
                ctx.request_layout();
                ctx.request_paint();
            }
        }
        self.child.lifecycle(ctx, event, &self.ty, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &Option<T>, _: &Option<T>, env: &Env) {
        self.child.update(ctx, &self.ty, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &Option<T>,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, &self.ty, env);
        self.child.set_origin(ctx, &self.ty, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &Option<T>, env: &Env) {
        self.child.paint(ctx, &self.ty, env)
    }
}

pub fn c_option_window<T: Travel + Serialize + DeserializeOwned + Data>(
    name: Option<&'static str>,
    on_change_fn: Option<OnChangeFn<T>>,
) -> impl Widget<Option<T>> {
    let child = Flex::column()
        .with_flex_child(CType::widget().scroll(), 1.0)
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Save")
                        .on_click(|ctx, _: &mut CType, _| {
                            ctx.submit_command(APPLY.to(Target::Window(ctx.window_id())));
                        })
                        .disabled_if(|data: &CType, _| {
                            T::deserialize(&mut ConfigDeserializer::new(data)).is_err()
                        }),
                )
                .with_child(Button::new("Cancel").on_click(|ctx, _: &mut CType, _| {
                    ctx.submit_command(CLOSE_WINDOW);
                })),
        );
    let mut buffer = CTypeBuffer::new(child);
    if let Some(name) = name {
        buffer.with_name(name)
    }
    if let Some(on_change) = on_change_fn {
        buffer.on_change(on_change)
    }
    buffer
}
