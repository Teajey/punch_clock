use crate::{
    error::{self, Result},
    fs::file_location_in_path_by_prefix,
};

pub struct Base {
    pub editor_path: String,
}

pub fn init() -> Result<Base> {
    let ctx_root = file_location_in_path_by_prefix(".punch_clock")?;

    std::env::set_current_dir(ctx_root)?;

    let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

    Ok(Base { editor_path })
}
