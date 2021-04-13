#![allow(dead_code)]

use clap::{AppSettings, Clap};
use config::{Config, ConfigEnum, InactiveBehavior};
use config_derive::Config;


#[derive(Config, Debug, Clone)]
struct TestConfig2 {
    #[config(default = "we23")]
    input: Vec<String>,
}

#[derive(Config, Clone, Debug)]
enum ConfigEnum2 {
    Hello,
    #[config( ty = "struct")]
    Blabla(TestConfig2),
}



#[derive(Config, Clone, Debug)]
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
    config_emi: ConfigEnum2,
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {}

fn main() {
    let mut app = TestConfig::build_app();
    println!("{:#?}", app);
    let s = TestConfig::parse_from_app(&app);
    println!("{:#?}", s);
    let x = s.unwrap().update_app(&mut app);
    println!("{:#?}", x);
}
