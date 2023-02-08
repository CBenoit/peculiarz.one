use anyhow::{Context as _, Result};

const HELP: &str = "\
cargo xtask

USAGE:
  cargo xtask [OPTIONS] [TASK]

FLAGS:
  -h, --help      Prints help information

TASKS:
  dist            Builds and package wasm modules
  start           Starts development server
  ci              Runs checks required on CI
  ci formatting   Checks formatting
  ci tests        Runs tests
  ci lints        Checks lints
  ci wasm         Ensures wasm modules are compatible for the web
  clean           Clean workspace
";

pub fn print_help() {
    println!("{HELP}");
}

pub enum Action {
    ShowHelp,
    Dist,
    Start,
    Ci,
    CiFormatting,
    CiTests,
    CiLints,
    CiWasm,
    Clean,
}

pub fn parse_args() -> Result<Action> {
    let mut args = pico_args::Arguments::from_env();

    let action = if args.contains(["-h", "--help"]) {
        Action::ShowHelp
    } else {
        match args.subcommand().context("Invalid subcommand")?.as_deref() {
            Some("dist") => Action::Dist,
            Some("start") => Action::Start,
            Some("ci") => match args.subcommand().context("Invalid CI action")?.as_deref() {
                Some("formatting") => Action::CiFormatting,
                Some("tests") => Action::CiTests,
                Some("lints") => Action::CiLints,
                Some("wasm") => Action::CiWasm,
                None => Action::Ci,
                Some(_) => anyhow::bail!("Unknown CI action"),
            },
            Some("clean") => Action::Clean,
            None | Some(_) => Action::ShowHelp,
        }
    };

    Ok(action)
}
