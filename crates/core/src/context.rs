use chrono::{FixedOffset, Local};

use crate::error::{self, Result};

pub struct Context<Tz> {
    pub editor_path: String,
    pub offset: Option<i32>,
    pub timezone: Tz,
    pub skip_hooks: bool,
}

impl Context<Local> {
    pub fn init(skip_hooks: bool) -> Result<Self> {
        let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

        Ok(Context {
            editor_path,
            offset: None,
            timezone: Local,
            skip_hooks,
        })
    }
}

impl Context<FixedOffset> {
    pub fn init_with_offset(skip_hooks: bool, offset: i32) -> Result<Self> {
        let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

        let timezone =
            FixedOffset::east_opt(offset * 3600).ok_or(error::Main::TimezoneOutOfRange(offset))?;

        Ok(Context {
            editor_path,
            offset: Some(offset),
            timezone,
            skip_hooks,
        })
    }
}
