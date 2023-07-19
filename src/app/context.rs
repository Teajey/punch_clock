use crate::{
    error::{self, Result},
    time::ContextTimeZone,
};

pub struct Context<Tz: ContextTimeZone> {
    pub editor_path: String,
    pub timezone: Tz,
}

impl<Tz: ContextTimeZone> Context<Tz> {
    pub fn init(timezone: Tz) -> Result<Self> {
        let editor_path = std::env::var("EDITOR").map_err(|_| error::Main::MissingEditorPath)?;

        Ok(Context {
            editor_path,
            timezone,
        })
    }
}
