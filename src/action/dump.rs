use chrono::Local;

use crate::{app::context, error::Result};

pub fn run(ctx: &context::Base) -> Result<()> {
    let record = ctx.record.borrow();
    println!("| {:<40} | {:<40} |", "Check-in", "Check-out");
    for _ in 0..69 {
        print!("=");
    }
    println!();
    for entry in record.get_entries() {
        let local_check_in_date = entry.check_in.with_timezone(&Local);
        let local_check_out_date = entry.get_check_out()?.with_timezone(&Local);
        println!(
            "| {:<40} | {:<40} |",
            local_check_in_date.format("%e %b %Y %r %Z"),
            local_check_out_date.format("%e %b %Y %r %Z"),
        );
    }

    if let Some(current_session) = record.get_current_session() {
        let local_current_session_date = current_session.with_timezone(&Local);
        println!(
            "| {:<40} | {:<40} |",
            local_current_session_date.format("%e %b %Y %r %Z"),
            "-",
        );
    }

    Ok(())
}
