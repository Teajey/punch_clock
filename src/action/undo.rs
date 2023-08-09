use chrono::{Local, Utc};
use dialoguer::Confirm;

use crate::{
    error::Result,
    record::{Latest, Record},
};

pub fn run(record: &mut Record<Utc>) -> Result<()> {
    let Some(dt) = record.clone_last_datetime()? else {
        println!("Record is empty; nothing to undo.");
        return Ok(());
    };

    if !Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to remove the most recent timestamp at {}?",
            dt.with_timezone(&Local).format("%c")
        ))
        .interact()?
    {
        return Ok(());
    }

    let dt = record.pop().expect("presence checked at start of function");
    println!(
        "Removed latest timestamp from {}",
        dt.with_timezone(&Local).format("%c")
    );
    match record.get_latest() {
        Latest::Entry(entry) => {
            let check_out = entry.get_check_out()?;
            println!(
                "Now clocked out since {}",
                check_out.with_timezone(&Local).format("%c")
            );
        }
        Latest::Current(current_session) => {
            println!(
                "Now clocked in since {}",
                current_session.with_timezone(&Local).format("%c")
            );
        }
        Latest::None => {
            println!("Record is now empty.");
        }
    }

    Ok(())
}
