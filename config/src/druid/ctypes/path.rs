use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::ctypes::path::CPath;
use crate::errors::Error;
use crate::traveller::{Travel, TravelPathConfig, Traveller};
use druid::im::Vector;
use druid::lens::Field;
use druid::widget::{Button, Flex, Label, Maybe, TextBox};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};
use druid::{FileDialogOptions, FileSpec};
use druid_widget_nursery::WidgetExt as _;
use serde::de::{EnumAccess, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl CPath {
    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}:")))
                    .lens(Self::name),
            )
            .with_child(TextBox::new().lens(Self::value.map(
                |value| {
                    match value {
                        Some(v) => v
                            .clone()
                            .into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| "".to_owned()),
                        None => "".to_owned(),
                    }
                },
                |value: &mut Option<PathBuf>, x| {
                    if x.is_empty() {
                        *value = None
                    } else {
                        *value = Some(PathBuf::from(x))
                    }
                },
            )))
            .with_child(
                Button::new("+")
                    .on_click(|ctx, data: &mut Self, _env| {
                        let open_dialog_options = FileDialogOptions::new()
                            .select_directories()
                            .name_label("Target")
                            .title("Choose a target for this lovely file")
                            .button_text("Export")
                            .default_name("MySavedFile.txt");
                        let open_dialog_options = match &data.path_config {
                            TravelPathConfig::AbsoluteExistDir => {
                                open_dialog_options.select_directories()
                            }
                            TravelPathConfig::AbsoluteExistFile(allowed) => open_dialog_options
                                .allowed_types(
                                    allowed
                                        .iter()
                                        .map(|file_spec| file_spec.clone().into())
                                        .collect(),
                                ),
                            _ => open_dialog_options,
                        };
                        ctx.submit_command(
                            druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options),
                        )
                    })
                    .on_command(
                        druid::commands::OPEN_FILE,
                        |ctx, file_info, data: &mut Self| {
                            // TODO doesn't work for multiple
                            data.value = Some(file_info.path.clone());
                            ctx.request_update();
                        },
                    ),
            )
    }
}
