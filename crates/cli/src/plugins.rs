use std::{ffi::OsStr, os::unix::fs::PermissionsExt as _, path::PathBuf};

use anyhow::Context as _;

const LOCAL_PLUGINS_DIR: &str = ".punch_clock/plugins/";

pub struct ReadDirPlugins {
    directory_reader: std::fs::ReadDir,
    file_match_prefix: &'static str,
}

impl ReadDirPlugins {
    fn try_next_entry(
        &self,
        entry: std::io::Result<std::fs::DirEntry>,
    ) -> anyhow::Result<Option<(String, PathBuf)>> {
        let entry = entry.context("reading directory entry")?;

        let path = entry.path();

        if path.is_dir() {
            return Ok(None);
        }

        let Some(file_name) = path
            .file_name()
            .and_then(OsStr::to_str)
            .map(ToOwned::to_owned)
        else {
            return Ok(None);
        };

        let Some(plugin_name) = file_name.strip_prefix(self.file_match_prefix) else {
            return Ok(None);
        };

        let mode = entry
            .metadata()
            .context("accessing metadata")?
            .permissions()
            .mode();

        let is_executable = mode & 0o111 != 0;
        if !is_executable {
            return Ok(None);
        }

        Ok(Some((plugin_name.to_owned(), path)))
    }
}

impl Iterator for ReadDirPlugins {
    type Item = anyhow::Result<(String, PathBuf)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entry = self.directory_reader.next()?;
            match self.try_next_entry(entry) {
                Ok(Some(item)) => return Some(Ok(item)),
                Ok(None) => (),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}

trait Executables {
    fn executables(self, file_match_prefix: &'static str) -> ReadDirPlugins;
}

impl Executables for std::fs::ReadDir {
    fn executables(self, file_match_prefix: &'static str) -> ReadDirPlugins {
        ReadDirPlugins {
            directory_reader: self,
            file_match_prefix,
        }
    }
}

pub struct Plugins<'a> {
    global_plugins: Box<dyn Iterator<Item = anyhow::Result<ReadDirPlugins>> + 'a>,
    local_plugins: Option<ReadDirPlugins>,
}

impl<'a> Plugins<'a> {
    pub fn try_new(path_var: &'a str) -> anyhow::Result<Self> {
        let global_plugins = std::env::split_paths(path_var).filter_map(|p| {
            if !p.exists() {
                return None;
            }
            let plugins = std::fs::read_dir(p)
                .context("reading a global plugins directory")
                .map(|directory_reader| directory_reader.executables("punch_clock-"));
            Some(plugins)
        });

        let local_plugins = if std::fs::exists(LOCAL_PLUGINS_DIR)
            .context("checking local plugin directory existence")?
        {
            let plugs = std::fs::read_dir(LOCAL_PLUGINS_DIR)
                .context("reading the local plugins directory")?
                .executables("");
            Some(plugs)
        } else {
            None
        };

        let plugins = Self {
            global_plugins: Box::new(global_plugins),
            local_plugins,
        };

        Ok(plugins)
    }

    pub fn lossy(&mut self) -> impl Iterator<Item = (String, PathBuf)> + '_ + use<'_, 'a> {
        self.flatten().flatten().flatten()
    }
}

impl Iterator for Plugins<'_> {
    type Item = anyhow::Result<ReadDirPlugins>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = self.global_plugins.next() {
            return Some(entry);
        }

        self.local_plugins.take().map(Ok)
    }
}

pub fn try_iter(path_var: &str) -> anyhow::Result<Plugins<'_>> {
    let plugins = Plugins::try_new(path_var)?;
    Ok(plugins)
}
