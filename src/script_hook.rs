use std::{path::PathBuf, process::Command};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

fn run_in_current_dir(name: &str) -> Result<(), Error> {
    let hook_file_name = format!("{}{}", "./", name);
    let hook_file_path = PathBuf::from(&hook_file_name);
    if !hook_file_path.exists() {
        return Ok(());
    }

    let cmd = Command::new(hook_file_name).output()?;

    if !cmd.status.success() {
        let stderr_text = String::from_utf8_lossy(&cmd.stderr);
        eprintln!("'{name}' hook failed. Dumping stderr:");
        eprintln!("{stderr_text}");
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
