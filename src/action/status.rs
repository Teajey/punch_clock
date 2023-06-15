use chrono::Utc;

use crate::{
    error::Result,
    record::{Latest, Record},
    time::Ago,
};

pub fn run(record: &Record<Utc>) -> Result<()> {
    match record.get_latest() {
        Latest::Current(current_session) => {
            let since = current_session.signed_duration_since(Utc::now());
            let ago = Ago(since);

            println!("Currently clocked in ({ago})");
        }
        Latest::Entry(last_entry) => {
            let since = last_entry
                .get_check_out()?
                .signed_duration_since(Utc::now());
            let ago = Ago(since);

            println!("Currently clocked out ({ago})");
        }
        Latest::None => println!("No clock in/out records have been created."),
    };

    Ok(())
}
