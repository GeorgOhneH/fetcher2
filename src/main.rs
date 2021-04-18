#![allow(dead_code)]

use clap::{AppSettings, Clap};
use config::{Config, ConfigEnum, InactiveBehavior, CStruct};
use config_derive::Config;
use serde::{Serialize, Deserialize};
use serde_yaml::Value;
mod sites;
mod queue;
mod session;
mod errors;


#[derive(Config, Debug, Clone, Serialize)]
struct TestConfig2 {
    #[config(default = "we23")]
    input: Vec<String>,
}

#[derive(Config, Clone, Debug, Serialize)]
enum ConfigEnum2 {
    Hello,
    #[config( ty = "struct")]
    Blabla(TestConfig2),
}




#[derive(Config, Debug, Clone, Serialize)]
struct TestConfig {
    #[config(default = 0, min = -1, max = 2)]
    int: isize,
    #[config(default = 0, max = 2)]
    option_int: Option<isize>,

    #[config(default = "1023")]
    option_str: Option<String>,

    #[config(
        default = "hello",
        inactive_behavior=InactiveBehavior::GrayOut,
        active_fn=|_app| false,
        gui_name= "ifg",
    )]
    str: String,

    /// This is a hint text
    #[config(default=true)]
    verbose3: bool,

    #[config(ty = "struct")]
    nested: TestConfig2,

    #[config(ty = "enum")]
    config_emi: Option<ConfigEnum2>,

    #[config(ty = "enum")]
    config_emi2: ConfigEnum2,
}



#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {}

fn main() {
    let mut app: CStruct = TestConfig::build_app();
    println!("{:#?}", app);
    match app.get_mut("config_emi2").unwrap().get_mut() {
        ::config::CTypes::Enum(cenum) => cenum.set_selected("Blabla".to_owned()),
        _ => panic!("iefs")
    };
    match app.get_mut("config_emi").unwrap().get_mut() {
        ::config::CTypes::Enum(cenum) => cenum.set_selected("Blabla".to_owned()),
        _ => panic!("iefs")
    };
    let mut s: TestConfig = TestConfig::parse_from_app(&app).unwrap();

    let parsed = serde_yaml::to_string(&s).unwrap();
    println!("{:#?}", s);
    println!("{}", parsed);

    let r = app.load_from_string(&parsed);
    println!("{:#?}", r);


    let x = s.update_app(&mut app);
    println!("{:#?}", x);
}
