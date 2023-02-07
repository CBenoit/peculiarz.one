mod cli;
mod section;
mod tasks;

use std::path::{Path, PathBuf};

use anyhow::Result;
use xshell::Shell;

use crate::cli::Action;

fn main() -> Result<()> {
    let action = match cli::parse_args() {
        Ok(action) => action,
        Err(e) => {
            cli::print_help();
            return Err(e);
        }
    };

    let sh = Shell::new()?;

    sh.change_dir(project_root());

    match action {
        Action::ShowHelp => cli::print_help(),
        Action::Dist => tasks::dist(&sh)?,
        Action::Start => {
            tasks::dist(&sh)?;
            tasks::start(&sh)?;
        }
        Action::Ci => {
            tasks::check_formatting(&sh)?;
            tasks::run_tests(&sh)?;
            tasks::check_lints(&sh)?;
            tasks::check_wasm(&sh)?;
        }
        Action::CiFormatting => tasks::check_formatting(&sh)?,
        Action::CiTests => tasks::run_tests(&sh)?,
        Action::CiLints => tasks::check_lints(&sh)?,
        Action::CiWasm => tasks::check_wasm(&sh)?,
        Action::Clean => tasks::clean_workspace(&sh)?,
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
