use crate::{
    error::{self, Result},
    fs::file_location_in_path_by_prefix,
    time::ContextTimeZone,
};

pub struct Context<Tz: ContextTimeZone> {
    pub editor_path: String,
    pub timezone: Tz,
}

impl<Tz: ContextTimeZone> Context<Tz> {
    pub fn init(timezone: Tz) -> Result<Self> {
        let ctx_root = file_location_in_path_by_prefix(".punch_clock")?;

        std::env::set_current_dir(ctx_root)?;

        let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

        Ok(Context {
            editor_path,
            timezone,
        })
    }
}
