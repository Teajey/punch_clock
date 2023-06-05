use chrono::Local;

use crate::app::context;

pub fn run(ctx: &context::Base) {
    println!("| {:<40} | {:<20} |", "Date", "Event");
    for _ in 0..69 {
        print!("=");
    }
    println!();
    for entry in &ctx.record.borrow().0 {
        let local_date = entry.date.with_timezone(&Local);
        println!(
            "| {:<40} | {:<20} |",
            local_date.format("%e %b %Y %r %Z"),
            // Seems weird that I need to use `to_string` for correct formatting...?
            entry.event.to_string()
        );
    }
}
