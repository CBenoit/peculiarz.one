use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context as _, Result};
use xshell::{cmd, Shell};

const HELP: &str = "\
cargo xtask

USAGE:
  cargo xtask [OPTIONS] [TASK]

FLAGS:
  -h, --help      Prints help information

TASKS:
  dist            …
  start           Starts development server
  ci              Runs checks required on CI
  ci formatting   Checks formatting
  ci tests        Runs tests
  ci lints        Checks lints
  ci wasm         Ensures wasm modules are compatible for the web
  clean           Clean workspace
";

const WASM_PACKAGES: &[&str] = &["bread-world", "knowledge"];

fn main() -> Result<()> {
    let action = match parse_args() {
        Ok(action) => action,
        Err(e) => {
            println!("{HELP}");
            return Err(e);
        }
    };

    let sh = Shell::new()?;

    sh.change_dir(project_root());

    match action {
        Action::ShowHelp => println!("{HELP}"),
        Action::Dist => dist(&sh)?,
        Action::Start => {
            dist(&sh)?;
            start(&sh)?;
        }
        Action::Ci(CiAction::All) => {
            check_formatting(&sh)?;
            run_tests(&sh)?;
            check_lints(&sh)?;
            check_wasm(&sh)?;
        }
        Action::Ci(CiAction::Formatting) => check_formatting(&sh)?,
        Action::Ci(CiAction::Tests) => run_tests(&sh)?,
        Action::Ci(CiAction::Lints) => check_lints(&sh)?,
        Action::Ci(CiAction::Wasm) => check_wasm(&sh)?,
        Action::Clean => clean_workspace(&sh)?,
    }

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn dist(sh: &Shell) -> Result<()> {
    use wasm_bindgen_cli_support::Bindgen;

    let _s = Section::new("DIST");

    cmd!(sh, "rustup target add wasm32-unknown-unknown").run()?;

    let dist_dir = Path::new("dist/");
    sh.create_dir(dist_dir)?;

    for package in WASM_PACKAGES {
        let _s = Section::new(package);

        cmd!(
            sh,
            "cargo build --release --locked --target wasm32-unknown-unknown --package {package}"
        )
        .run()?;

        let input_path = PathBuf::from(format!("target/wasm32-unknown-unknown/release/{package}.wasm"));

        let mut output = Bindgen::new()
            .input_path(input_path)
            .out_name(package)
            .web(true)
            .unwrap()
            .debug(false)
            .generate_output()
            .context("Couldn’t generate WASM bindgen file")?;

        let js = output.js();
        let js_path = dist_dir.join(package).with_extension("js");
        std::fs::write(&js_path, js).with_context(|| format!("Cannot write js file at {}", js_path.display()))?;

        let wasm = output.wasm_mut().emit_wasm();
        let wasm_path = dist_dir.join(package).with_extension("wasm");
        std::fs::write(&wasm_path, wasm)
            .with_context(|| format!("Cannot write WASM file at {}", wasm_path.display()))?;

        let optimized_wasm_path = dist_dir.join(format!("{package}-opt")).with_extension("wasm");
        cmd!(sh, "wasm-opt -Os {wasm_path} -o {optimized_wasm_path}").run()?;

        sh.copy_file(js_path, format!("assets/app/{package}.js"))?;
        sh.copy_file(optimized_wasm_path, format!("assets/app/{package}.wasm"))?;
    }

    Ok(())
}

fn start(sh: &Shell) -> Result<()> {
    let _s = Section::new("START");
    cmd!(sh, "cargo run").run()?;
    Ok(())
}

fn check_formatting(sh: &Shell) -> Result<()> {
    let _s = Section::new("FORMATTING");

    let output = cmd!(sh, "cargo fmt --all -- --check").ignore_status().output()?;

    if !output.status.success() {
        anyhow::bail!("Bad formatting, please run 'cargo +stable fmt --all'");
    }

    println!("All good!");

    Ok(())
}

fn run_tests(sh: &Shell) -> Result<()> {
    let _s = Section::new("TESTS");
    cmd!(sh, "cargo test --workspace --locked").run()?;
    println!("All good!");
    Ok(())
}

fn check_lints(sh: &Shell) -> Result<()> {
    let _s = Section::new("LINTS");
    cmd!(sh, "cargo clippy --workspace --locked -- -D warnings").run()?;
    println!("All good!");
    Ok(())
}

fn check_wasm(sh: &Shell) -> Result<()> {
    let _s = Section::new("WASM-CHECK");

    cmd!(sh, "rustup target add wasm32-unknown-unknown").run()?;

    for package in WASM_PACKAGES {
        let _s = Section::new(package);

        cmd!(
            sh,
            "cargo build --locked --target wasm32-unknown-unknown --package {package}"
        )
        .run()?;

        let output = cmd!(sh, "wasm2wat ./target/wasm32-unknown-unknown/debug/{package}.wasm").output()?;
        let stdout = std::str::from_utf8(&output.stdout).context("wasm2wat output is not valid UTF-8")?;

        if stdout.contains("import \"env\"") {
            anyhow::bail!("Found undefined symbols in generated wasm file");
        }
    }

    println!("All good!");

    Ok(())
}

fn clean_workspace(sh: &Shell) -> Result<()> {
    let _s = Section::new("CLEAN");

    cmd!(sh, "cargo clean").run()?;

    sh.remove_path("dist")?;

    for package in WASM_PACKAGES {
        sh.remove_path(format!("assets/app/{package}.js"))?;
        sh.remove_path(format!("assets/app/{package}.wasm"))?;
    }

    Ok(())
}

enum Action {
    ShowHelp,
    Dist,
    Start,
    Ci(CiAction),
    Clean,
}

enum CiAction {
    All,
    Formatting,
    Tests,
    Lints,
    Wasm,
}

fn parse_args() -> Result<Action> {
    let mut args = pico_args::Arguments::from_env();

    let action = if args.contains(["-h", "--help"]) {
        Action::ShowHelp
    } else {
        match args.subcommand().context("Invalid subcommand")?.as_deref() {
            Some("dist") => Action::Dist,
            Some("start") => Action::Start,
            Some("ci") => match args.subcommand().context("Invalid CI action")?.as_deref() {
                Some("formatting") => Action::Ci(CiAction::Formatting),
                Some("tests") => Action::Ci(CiAction::Tests),
                Some("lints") => Action::Ci(CiAction::Lints),
                Some("wasm") => Action::Ci(CiAction::Wasm),
                None => Action::Ci(CiAction::All),
                Some(_) => anyhow::bail!("Unknown CI action"),
            },
            Some("clean") => Action::Clean,
            None | Some(_) => Action::ShowHelp,
        }
    };

    Ok(action)
}

struct Section {
    name: &'static str,
    start: Instant,
}

impl Section {
    fn new(name: &'static str) -> Section {
        println!("::group::{name}");
        let start = Instant::now();
        Section { name, start }
    }
}

impl Drop for Section {
    fn drop(&mut self) {
        println!("{}: {:.2?}", self.name, self.start.elapsed());
        println!("::endgroup::");
    }
}
