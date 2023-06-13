use chrono::Local;

use crate::{app::context, error::Result};

const DATE_FORMAT: &str = "%e %b %Y %I:%M%P %Z";

pub fn run(record: &context::Record) -> Result<()> {
    println!("| {:<40} | {:<40} |", "Check-in", "Check-out");
    for _ in 0..89 {
        print!("=");
    }
    println!();
    for entry in record.get_entries() {
        let local_check_in_date = entry.check_in.with_timezone(&Local);
        let local_check_out_date = entry.get_check_out()?.with_timezone(&Local);
        println!(
            "| {:<40} | {:<40} |",
            local_check_in_date.format(DATE_FORMAT),
            local_check_out_date.format(DATE_FORMAT),
        );
    }

    if let Some(current_session) = record.get_current_session() {
        let local_current_session_date = current_session.with_timezone(&Local);
        println!(
            "| {:<40} | {:<40} |",
            local_current_session_date.format(DATE_FORMAT),
            "-",
        );
    }

    Ok(())
}
