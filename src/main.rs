// (Full example with detailed comments in examples/01d_quick_example.rs)
//
// This example demonstrates clap's full 'custom derive' style of creating arguments which is the
// simplest method of use, but sacrifices some flexibility.
#![allow(dead_code)]

use clap::{AppSettings, Clap};
use config::{
    Config, ConfigArg, ConfigArgBool, ConfigArgString, InactiveBehavior,
    SupportedTypes,
};
use config_derive::Config;

#[derive(Config)]
#[config(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct TestConfig2 {
}

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Config)]
#[config(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
struct TestConfig {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    config: Option<bool>,
    /// Some input. Because this isn't an Option<T> it's required to be used
    #[config(default="hello")]
    input: String,
    /// A level of verbosity, and can be used multiple times
    #[config(min = 3, max = 4, active_fn = |app| false)]
    verbose: i32,
    /// help test

    #[config(
        gui_name = "blabla",
        inactive_behavior = InactiveBehavior::Hide,
    )]
    vec: Vec<TestConfig2>, //
}

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {}

fn main() {
    let a = TestConfig::build_app();
    println!("{:#?}", a)
}
