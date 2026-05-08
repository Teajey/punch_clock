use std::{
    path::PathBuf,
    process::{Child, ChildStdin, Command},
};

use anyhow::Context as _;

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

#[deprecated]
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

pub struct Hook {
    name: &'static str,
    command: Option<Command>,
}

pub struct Process {
    name: &'static str,
    child: Option<Child>,
}

impl Hook {
    pub fn try_new(name: &'static str) -> anyhow::Result<Self> {
        let mut hook = Hook {
            name,
            command: None,
        };

        let skip = std::env::var("PUNCH_CLOCK_SKIP_HOOKS")
            .ok()
            .and_then(|x| x.parse::<bool>().ok())
            .unwrap_or_default();

        if skip {
            return Ok(hook);
        }

        let current_dir = std::env::current_dir().context("getting current directory")?;
        let hook_file_path = current_dir.join(".punch_clock/hooks").join(name);
        if hook_file_path.exists() {
            let mut cmd = Command::new(&hook_file_path);
            cmd.args(std::env::args().skip(1));
            hook.command = Some(cmd);
        }

        Ok(hook)
    }

    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        let Some(command) = &mut self.command else {
            return self;
        };
        command.env(key, value);
        self
    }

    pub fn run(&mut self) {
        let Some(command) = &mut self.command else {
            return;
        };

        let status = match command.status() {
            Ok(status) => status,
            Err(err) => {
                eprintln!("Failed to execute hook '{}': {}", self.name, err);
                std::process::exit(-1);
            }
        };

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            eprintln!("'{}' hook exited with code {exit_code}", self.name);
            std::process::exit(exit_code);
        }
    }

    pub fn get_process(&mut self) -> anyhow::Result<Process> {
        let Some(command) = &mut self.command else {
            return Ok(Process {
                name: self.name,
                child: None,
            });
        };

        let child = command
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("spawning child process")?;

        Ok(Process {
            name: self.name,
            child: Some(child),
        })
    }
}

impl Process {
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        let Some(child) = &mut self.child else {
            return None;
        };
        child.stdin.take()
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let Some(child) = &mut self.child else {
            return Ok(());
        };

        let status = child.wait().context("waiting on child process")?;

        if !status.success() {
            let exit_code = status.code().unwrap_or(-1);
            println!("'{}' hook exited with code {exit_code}", self.name);
            std::process::exit(exit_code);
        }

        Ok(())
    }
}
