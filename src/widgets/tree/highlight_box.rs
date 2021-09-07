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

//! A widget with predefined size.

use std::f64::INFINITY;
use tracing::{instrument, trace, warn};

use druid::widget::prelude::*;
use druid::{Data, theme};

/// A widget with predefined size.
///
/// If given a child, this widget forces its child to have a specific width and/or height
/// (assuming values are permitted by this widget's parent). If either the width or height is not
/// set, this widget will size itself to match the child's size in that dimension.
///
/// If not given a child, SizedBox will try to size itself as close to the specified height
/// and width as possible given the parent's constraints. If height or width is not set,
/// it will be treated as zero.
pub struct ColourBox {
    pub selected: bool,
}

impl ColourBox {
    pub fn new() -> Self {
        Self {
            selected: false
        }
    }
}

impl Widget<()> for ColourBox {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &(), env: &Env) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint()
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &(), data: &(), env: &Env) {}

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &(), env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &(), env: &Env) {
        let rect = ctx.size().to_rect();
        if self.selected {
            ctx.fill(rect, &env.get(theme::PRIMARY_DARK));
        } else if ctx.is_hot() {
            ctx.fill(rect, &env.get(theme::PRIMARY_LIGHT));
        }
    }
}
