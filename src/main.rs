#![allow(dead_code)]

use async_std::channel;
mod errors;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;

use crate::settings::DownloadSettings;
use crate::template::{DownloadArgs, Template};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use config::Config;
use serde::Serialize;
use config_derive::Config;


// #[tokio::main]
// async fn main() {
//     let template = Template::new();
//     let session = crate::session::Session::new();
//     let dsettings = DownloadSettings {
//         username: "gshwan".to_owned(),
//         password: "".to_owned(),
//         save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
//         download_args: DownloadArgs {
//             allowed_extensions: vec![],
//             forbidden_extensions: vec![],
//         },
//     };
//     template.run_root(session, &dsettings).await.unwrap()
// }

#[derive(Clone, Serialize)]
pub struct SiteStorage {
    files: HashMap<PathBuf, String>
}

impl ::config::Config for SiteStorage {
    fn build_app() -> ::config::CStruct {
        ::config::CStructBuilder::new()
            .arg(
                ::config::CKwargBuilder::new(
                    "files".to_string(),
                    ::config::CTypes::HashMap(
                        ::config::CHashMapBuilder::new(
                            || ::config::CTypes::Path(::config::CPathBuilder::new().build()),
                            || ::config::CTypes::String(::config::CStringBuilder::new().build()),
                        )
                            .build(),
                    ),
                )
                    .required(false)
                    .build(),
            )
            .build()
    }
    fn parse_from_app(app: &::config::CStruct) -> Result<Self, ::config::RequiredError> {
        let files = match {
            let a: Result<HashMap<PathBuf, String>, ::config::RequiredError> = match app
                .get_ty(&"files".to_string())
                .unwrap()
            {
                ::config::CTypes::HashMap(cmap) => cmap
                    .get()
                    .iter()
                    .map(|(keytype, valuetype)| {
                        let x = match {
                            match keytype {
                                ::config::CTypes::Path(cpath) => Ok(cpath.get()),
                                _ => panic!("This should never happen"),
                            }
                        } {
                            Ok(value) => match value {
                                Some(x) => Ok(x.clone()),
                                None => {
                                    Err(::config::RequiredError::new("files", "Must be Option?"))
                                }
                            },
                            Err(err) => Err(err),
                        }?;
                        let y = match {
                            match valuetype {
                                ::config::CTypes::String(value_arg) => Ok(value_arg.get()),
                                _ => panic!("This should never happen"),
                            }
                        } {
                            Ok(value) => match value {
                                Some(x) => Ok(x.clone()),
                                None => {
                                    Err(::config::RequiredError::new("files", "Must be Option?"))
                                }
                            },
                            Err(err) => Err(err),
                        }?;
                        Ok((x, y))
                    })
                    .collect(),
                _ => panic!("This should never happen"),
            };
            match a {
                Ok(value) => Ok(Some(value)),
                Err(err) => Err(err),
            }
        } {
            Ok(value) => match value {
                Some(x) => Ok(x.clone()),
                None => Err(::config::RequiredError::new("files", "Must be Option?")),
            },
            Err(err) => Err(err),
        };
        if let Err(err) = files {
            return Err(err);
        };
        Ok(Self {
            files: files.unwrap(),
        })
    }
    fn update_app(self, app: &mut ::config::CStruct) -> Result<(), ::config::InvalidError> {
        let results: Vec<Result<(), ::config::InvalidError>> = vec![{
            let cmap = match app.get_ty_mut("files").unwrap() {
                ::config::CTypes::HashMap(ref mut cmap) => cmap,
                _ => panic!("This should never happen"),
            };
            let a: Result<HashMap<::config::CTypes, ::config::CTypes>, ::config::InvalidError> =
                self.files
                    .into_iter()
                    .map(|(key, value)| {
                        let mut key_temp = cmap.get_key();
                        let mut value_temp = cmap.get_value();
                        {
                            match key_temp {
                                ::config::CTypes::Path(ref mut cpath) => cpath.set(key),
                                _ => panic!("This should never happen"),
                            }
                        }?;
                        {
                            match value_temp {
                                ::config::CTypes::String(ref mut cstring) => {
                                    cstring.set(value);
                                    Ok(())
                                }
                                _ => panic!("This should never happen"),
                            }
                        }?;
                        Ok((key_temp, value_temp))
                    })
                    .collect();
            match a {
                Ok(map) => cmap.set(map),
                Err(err) => Err(err),
            }
        }];
        for result in results {
            if let Err(err) = result {
                return Err(err);
            }
        }
        Ok(())
    }
}







fn main() {}


