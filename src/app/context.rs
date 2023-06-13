use crate::error::{self, Result};

pub struct Base {
    pub editor_path: String,
}

pub fn load() -> Result<Base> {
    let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

    Ok(Base { editor_path })
}
