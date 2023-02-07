use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use xshell::{cmd, Shell};

use crate::section::Section;

const WASM_PACKAGES: &[&str] = &["bread-world", "knowledge"];
const CARGO: &str = env!("CARGO");

pub fn dist(sh: &Shell) -> Result<()> {
    use wasm_bindgen_cli_support::Bindgen;

    let _s = Section::new("DIST");

    cmd!(sh, "rustup target add wasm32-unknown-unknown").run()?;

    let dist_dir = Path::new("dist/");
    sh.create_dir(dist_dir)?;

    let app_dir = Path::new("assets/app/");
    sh.create_dir(app_dir)?;

    for package in WASM_PACKAGES {
        println!("Package {package}");

        cmd!(
            sh,
            "{CARGO} build --release --locked --target wasm32-unknown-unknown --package {package}"
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

        sh.copy_file(js_path, app_dir.join(package).with_extension("js"))?;
        sh.copy_file(optimized_wasm_path, app_dir.join(package).with_extension("wasm"))?;
    }

    Ok(())
}

pub fn start(sh: &Shell) -> Result<()> {
    let _s = Section::new("START");
    cmd!(sh, "{CARGO} run").run()?;
    Ok(())
}

pub fn check_formatting(sh: &Shell) -> Result<()> {
    let _s = Section::new("FORMATTING");

    let output = cmd!(sh, "{CARGO} fmt --all -- --check").ignore_status().output()?;

    if !output.status.success() {
        anyhow::bail!("Bad formatting, please run '{CARGO} +stable fmt --all'");
    }

    println!("All good!");

    Ok(())
}

pub fn run_tests(sh: &Shell) -> Result<()> {
    let _s = Section::new("TESTS");
    cmd!(sh, "{CARGO} test --workspace --locked").run()?;
    println!("All good!");
    Ok(())
}

pub fn check_lints(sh: &Shell) -> Result<()> {
    let _s = Section::new("LINTS");
    cmd!(sh, "{CARGO} clippy --workspace --locked -- -D warnings").run()?;
    println!("All good!");
    Ok(())
}

pub fn check_wasm(sh: &Shell) -> Result<()> {
    let _s = Section::new("WASM-CHECK");

    cmd!(sh, "rustup target add wasm32-unknown-unknown").run()?;

    for package in WASM_PACKAGES {
        println!("Check {package}");

        cmd!(
            sh,
            "{CARGO} build --locked --target wasm32-unknown-unknown --package {package}"
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

pub fn clean_workspace(sh: &Shell) -> Result<()> {
    let _s = Section::new("CLEAN");

    cmd!(sh, "{CARGO} clean").run()?;

    sh.remove_path("dist")?;

    for package in WASM_PACKAGES {
        sh.remove_path(format!("assets/app/{package}.js"))?;
        sh.remove_path(format!("assets/app/{package}.wasm"))?;
    }

    Ok(())
}
