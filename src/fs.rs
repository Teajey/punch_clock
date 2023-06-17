use std::path::{Path, PathBuf};

use crate::error::{Main, Result};

pub fn file_location_in_path_by_prefix(prefix: &str) -> Result<PathBuf> {
    fn recurse(prefix: &str, current_dir: &Path) -> Result<PathBuf> {
        let files = std::fs::read_dir(current_dir)?.collect::<Result<Vec<_>, _>>()?;

        let file_names = files
            .into_iter()
            .map(|f| f.file_name().into_string())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Main::OsStringParseFail)?;

        let prefix_matched = file_names.into_iter().any(|f| f.starts_with(prefix));

        if prefix_matched {
            Ok(current_dir.to_path_buf())
        } else if let Some(dir) = current_dir.parent() {
            recurse(prefix, dir)
        } else {
            Err(Main::NoPrefixInPath(prefix.to_owned()))
        }
    }

    recurse(prefix, &std::env::current_dir()?)
}
