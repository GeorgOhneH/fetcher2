// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Simple list view widget.

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::f64;
use std::sync::Arc;

use druid::im::{OrdMap, Vector};

use druid::kurbo::{Point, Rect, Size};

use crate::{CItem, CVec};
use druid::{
    widget::Axis, BoxConstraints, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, UpdateCtx, Widget, WidgetPod,
};

/// A list widget for a variable-size collection of items.
pub struct List {
    closure: Box<dyn Fn() -> Box<dyn Widget<CItem>>>,
    children: Vec<WidgetPod<CItem, Box<dyn Widget<CItem>>>>,
    axis: Axis,
    spacing: KeyOrValue<f64>,
}

impl List {
    /// Create a new list widget. Closure will be called every time when a new child
    /// needs to be constructed.
    pub fn new<W: Widget<CItem> + 'static>(closure: impl Fn() -> W + 'static) -> Self {
        List {
            closure: Box::new(move || Box::new(closure())),
            children: Vec::new(),
            axis: Axis::Vertical,
            spacing: KeyOrValue::Concrete(0.),
        }
    }

    /// Sets the widget to display the list horizontally, not vertically.
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    /// Set the spacing between elements.
    pub fn with_spacing(mut self, spacing: impl Into<KeyOrValue<f64>>) -> Self {
        self.spacing = spacing.into();
        self
    }

    /// Set the spacing between elements.
    pub fn set_spacing(&mut self, spacing: impl Into<KeyOrValue<f64>>) -> &mut Self {
        self.spacing = spacing.into();
        self
    }

    /// When the widget is created or the data changes, create or remove children as needed
    ///
    /// Returns `true` if children were added or removed.
    fn update_child_count(&mut self, data: &CVec, _env: &Env) -> bool {
        let len = self.children.len();
        match len.cmp(&data.get().len()) {
            Ordering::Greater => self.children.truncate(data.get().len()),
            Ordering::Less => {
                for _ in 0..data.get().len() - len {
                    let child = WidgetPod::new((self.closure)());
                    self.children.push(child);
                }
            }
            Ordering::Equal => (),
        }
        len != data.get().len()
    }
}

impl Widget<CVec> for List {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CVec, env: &Env) {
        for (child_data, child) in data.get_mut().iter_mut().zip(self.children.iter_mut()) {
            child.event(ctx, event, child_data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CVec, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if self.update_child_count(data, env) {
                ctx.children_changed();
            }
        }

        for (child_data, child) in data.get().iter().zip(self.children.iter_mut()) {
            child.lifecycle(ctx, event, child_data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &CVec, data: &CVec, env: &Env) {
        // we send update to children first, before adding or removing children;
        // this way we avoid sending update to newly added children, at the cost
        // of potentially updating children that are going to be removed.
        // dbg!(_old_data, data);
        for (child_data, child) in data.get().iter().zip(self.children.iter_mut()) {
            child.update(ctx, child_data, env);
        }

        if self.update_child_count(data, env) {
            dbg!("WEOGOF(GFEUIFIEGUFOIEGFUEUF");
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &CVec, env: &Env) -> Size {
        let axis = self.axis;
        let spacing = self.spacing.resolve(env);
        let mut minor = axis.minor(bc.min());
        let mut major_pos = 0.0;
        let mut paint_rect = Rect::ZERO;
        let child_bc = match axis {
            Axis::Horizontal => BoxConstraints::new(
                Size::new(0., bc.min().height),
                Size::new(f64::INFINITY, bc.max().height),
            ),
            Axis::Vertical => BoxConstraints::new(
                Size::new(bc.min().width, 0.),
                Size::new(bc.max().width, f64::INFINITY),
            ),
        };
        for (child_data, child) in data.get().iter().zip(self.children.iter_mut()) {
            let child_size = child.layout(ctx, &child_bc, child_data, env);
            let child_pos: Point = axis.pack(major_pos, 0.).into();
            child.set_origin(ctx, child_data, env, child_pos);
            paint_rect = paint_rect.union(child.paint_rect());
            minor = minor.max(axis.minor(child_size));
            major_pos += axis.major(child_size) + spacing;
        }

        // correct overshoot at end.
        major_pos -= spacing;

        let my_size = bc.constrain(Size::from(axis.pack(major_pos, minor)));
        let insets = paint_rect - my_size.to_rect();
        ctx.set_paint_insets(insets);
        my_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CVec, env: &Env) {
        for (child_data, child) in data.get().iter().zip(self.children.iter_mut()) {
            child.paint(ctx, child_data, env);
        }
    }
}
