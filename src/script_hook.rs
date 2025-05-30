use std::{path::PathBuf, process::Command};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn run_in_current_dir(name: &str) -> Result<(), Error> {
    let hook_file_name = format!("./{name}");
    let hook_file_path = PathBuf::from(&hook_file_name);
    if !hook_file_path.exists() {
        return Ok(());
    }

    let status = Command::new(hook_file_name).status()?;

    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);
        println!("'{name}' hook exited with code {exit_code}");
        std::process::exit(exit_code);
    }

    Ok(())
}

pub fn run(name: &str) -> Result<(), Error> {
    let origin_dir = std::env::current_dir()?;

    let hooks_dir = PathBuf::from(".punch_clock/hooks");
    if !hooks_dir.exists() {
        return Ok(());
    }

    std::env::set_current_dir(hooks_dir)?;

    let result = run_in_current_dir(name);

    std::env::set_current_dir(origin_dir)?;

    result
}
