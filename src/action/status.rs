use chrono::Utc;

use crate::{
    app::context::{self, RecordLatest},
    error::{Ago, Result},
};

pub fn run(ctx: &context::Base) -> Result<()> {
    match ctx.record.borrow().get_latest() {
        RecordLatest::Current(current_session) => {
            let since = current_session.signed_duration_since(Utc::now());
            let ago = Ago(since);

            println!("Currently clocked in ({ago})");
        }
        RecordLatest::Entry(last_entry) => {
            let since = last_entry
                .get_check_out()?
                .signed_duration_since(Utc::now());
            let ago = Ago(since);

            println!("Currently clocked out ({ago})");
        }
        RecordLatest::None => println!("No clock in/out records have been created."),
    };

    Ok(())
}
