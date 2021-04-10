// (Full example with detailed comments in examples/01d_quick_example.rs)
//
// This example demonstrates clap's full 'custom derive' style of creating arguments which is the
// simplest method of use, but sacrifices some flexibility.
#![allow(dead_code)]

use clap::{AppSettings, Clap};
use config::{Config, InactiveBehavior};
use config_derive::Config;

#[derive(Config, Debug, Clone)]
struct TestConfig3 {
    #[config(default = "1")]
    input: String,
}

#[derive(Config, Debug, Clone)]
struct TestConfig2 {
    #[config()]
    input: Option<TestConfig3>,
}

#[derive(Config, Clone, Debug)]
struct TestConfig {
    #[config(default = 0)]
    config: isize,

    #[config()]
    input: Option<String>,
    //
    #[config(default = 4, min = 3, max = 4, active_fn = |app| false)]
    verbose: isize,

    #[config(
        gui_name = "blabla",
        inactive_behavior = InactiveBehavior::Hide,
    )]
    vec: Vec<TestConfig2>,

    #[config(checked = true)]
    s: Option<TestConfig3>,
    ss: TestConfig2,
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
