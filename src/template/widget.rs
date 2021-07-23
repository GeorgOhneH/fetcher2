use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, ExtEventSink, Rect, Selector, SingleUse, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::nodes::node_widget::{NodeWidget, SELECT};
use crate::template::nodes::root_widget::{RootNodeData, RootNodeWidget};
use crate::template::Template;
use crate::widgets::{Split, SplitOrBox};
use crate::Result;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::cmp::max;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Data)]
pub struct TemplateData {
    pub root: RootNodeData,
    pub selected: Option<Vector<usize>>,
}

pub struct TemplateWidget {
    header_count: usize,
    header: WidgetPod<TemplateData, Split<TemplateData>>,
    root: WidgetPod<RootNodeData, RootNodeWidget>,
    section_bc: Option<Vec<(f64, f64)>>,
}

impl TemplateWidget {
    pub fn new(
        mut root: RootNodeWidget,
        header_widgets: Vec<Box<dyn Widget<TemplateData>>>,
    ) -> Self {
        let header_count = header_widgets.len();
        root.set_section_num(header_count);
        let split = Self::create_split_widget(header_widgets);
        Self {
            header_count,
            header: WidgetPod::new(split),
            root: WidgetPod::new(root),
            section_bc: None,
        }
    }

    fn updated_selection(&mut self, selection: &[usize]) {
        self.root.widget_mut().updated_selection(selection)
    }

    fn create_split_widget(
        header_widgets: Vec<Box<dyn Widget<TemplateData>>>,
    ) -> Split<TemplateData> {
        match header_widgets.len() {
            0 | 1 => panic!("Header must be greater than one"),
            2 => {
                let mut iter = header_widgets.into_iter();
                Split::columns(iter.next().unwrap(), SplitOrBox::Box(iter.next().unwrap()))
                    .draggable(true)
                    .solid_bar(true)
                    .bar_size(2.)
                    .min_bar_area(2.)
            }
            len => {
                let mut iter = header_widgets.into_iter();
                Split::columns(
                    iter.next().unwrap(),
                    SplitOrBox::Split(Box::new(Self::create_split_widget(iter.collect()))),
                )
                .draggable(true)
                .split_point(1. / len as f64)
                .solid_bar(true)
                .bar_size(2.)
                .min_bar_area(2.)
            }
        }
    }

    fn get_sections_bc(&self) -> Vec<(f64, f64)> {
        Self::_get_sections_bc(self.header.widget())
    }

    fn _get_sections_bc(split: &Split<TemplateData>) -> Vec<(f64, f64)> {
        let (child1_info, child2_info) = split.child_info.as_ref().unwrap();
        match split.child2.widget() {
            SplitOrBox::Split(split) => {
                let mut r = vec![(child1_info.point.x, child1_info.bc.max().width)];
                r.append(
                    &mut Self::_get_sections_bc(split)
                        .iter()
                        .map(|(point, width)| (point + child2_info.point.x, *width))
                        .collect::<Vec<_>>(),
                );
                r
            }
            SplitOrBox::Box(widget) => {
                vec![
                    (child1_info.point.x, child1_info.bc.max().width),
                    (child2_info.point.x, child2_info.bc.max().width),
                ]
            }
        }
    }
}

// Implement the Widget trait for Tree
impl Widget<TemplateData> for TemplateWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut TemplateData, env: &Env) {
        // eprintln!("{:?}", event);
        match event {
            Event::Notification(notif) => {
                if let Some(select) = notif.get(SELECT) {
                    ctx.set_handled();
                    let mut sel = select.take().unwrap();
                    sel.reverse();
                    self.updated_selection(&sel[..]);
                    data.selected = Some(sel.into());
                }
                return;
            }
            _ => (),
        }
        self.header.event(ctx, event, data, env);
        self.root.event(ctx, event, &mut data.root, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &TemplateData,
        env: &Env,
    ) {
        self.header.lifecycle(ctx, event, &data, env);
        self.root.lifecycle(ctx, event, &data.root, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &TemplateData,
        data: &TemplateData,
        env: &Env,
    ) {
        self.header.update(ctx, &data, env);
        self.root.update(ctx, &data.root, env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &TemplateData,
        env: &Env,
    ) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let min_width = bc.min().width;
        let max_width = bc.max().width;

        let header_size = self.header.layout(
            ctx,
            &BoxConstraints::new(
                Size::new(min_width, basic_size),
                Size::new(max_width, basic_size),
            ),
            &data,
            env,
        );
        self.header.set_origin(ctx, &data, env, Point::ORIGIN);

        self.section_bc = Some(self.get_sections_bc());
        self.root
            .widget_mut()
            .set_section_bc(self.section_bc.clone().unwrap());

        let root_size = self.root.layout(ctx, bc, &data.root, env);
        self.root
            .set_origin(ctx, &data.root, env, Point::new(0., header_size.height));
        let total_size = Size::new(
            header_size.width.max(root_size.width),
            header_size.height + root_size.height,
        );
        bc.constrain(total_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &TemplateData, env: &Env) {
        let background_color = env.get(theme::BACKGROUND_LIGHT);
        let background_color2 = env.get(theme::FOREGROUND_DARK);
        let clip_rect = ctx.size().to_rect();
        ctx.fill(clip_rect, &background_color);
        self.header.paint(ctx, &data, env);
        let root_rect = self.root.layout_rect();
        self.root.paint(ctx, &data.root, env);
        for i in 0..self.section_bc.as_ref().unwrap().len() - 1 {
            let (x_ahead, _) = self.section_bc.as_ref().unwrap()[i + 1];
            let (x_before, width) = self.section_bc.as_ref().unwrap()[i];
            let r = Rect::new(x_before + width, root_rect.y0, x_ahead, root_rect.y1);
            ctx.fill(r, &background_color2);
        }
    }
}
