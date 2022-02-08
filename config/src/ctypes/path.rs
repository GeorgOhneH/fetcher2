use std::path::{Path, PathBuf};

use druid::im::Vector;
use druid::widget::{Button, Flex, Label, Maybe, TextBox};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};
use druid::{FileDialogOptions, FileSpec};
use druid_widget_nursery::WidgetExt as _;
use crate::errors::Error;

#[derive(Debug, Clone)]
pub enum CPathTypes {
    Folder,
    File(Vector<FileSpec>),
    Any,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CPath {
    #[data(same_fn = "PartialEq::eq")]
    value: Option<PathBuf>,
    #[data(ignore)]
    ty: CPathTypes,
    #[data(ignore)]
    name: Option<String>,
    #[data(ignore)]
    must_exist: bool,
    must_absolute: bool,
}

impl CPath {
    fn new() -> Self {
        Self {
            value: None,
            ty: CPathTypes::Any,
            name: None,
            must_exist: false,
            must_absolute: false,
        }
    }

    pub fn is_valid(&self, path: &Path) -> Result<(), Error> {
        todo!()
        // if self.must_absolute && path.is_relative() {
        //     return Err(InvalidError::new("Path must be absolute"));
        // }
        // if self.must_absolute && self.must_exist && !path.exists() {
        //     return Err(InvalidError::new("Path must exist"));
        // }
        // match &self.ty {
        //     CPathTypes::Any => Ok(()),
        //     CPathTypes::Folder => {
        //         if path.is_dir() {
        //             Ok(())
        //         } else {
        //             Err(InvalidError::new("Path must be a folder"))
        //         }
        //     }
        //     CPathTypes::File(extensions) => {
        //         if !path.is_file() {
        //             return Err(InvalidError::new("Path must be a file"));
        //         }
        //
        //         if !extensions.is_empty() {
        //             if let Some(file_extension) = path.extension() {
        //                 let ex = file_extension
        //                     .to_str()
        //                     .ok_or_else(|| InvalidError::new("Path must have a extension"))?;
        //                 if extensions
        //                     .iter()
        //                     .any(|file_spec| file_spec.extensions.contains(&ex))
        //                 {
        //                     Ok(())
        //                 } else {
        //                     Err(InvalidError::new("Path must have a extension"))
        //                 }
        //             } else {
        //                 Err(InvalidError::new("Path must have a extension"))
        //             }
        //         } else {
        //             Ok(())
        //         }
        //     }
        // }
    }


    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":"))
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
                        let open_dialog_options = match &data.ty {
                            CPathTypes::Any => open_dialog_options,
                            CPathTypes::Folder => open_dialog_options.select_directories(),
                            CPathTypes::File(allowed) => open_dialog_options
                                .allowed_types(allowed.clone().into_iter().collect()),
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
