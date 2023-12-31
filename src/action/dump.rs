use crate::{error::Result, record::Record, time::ContextTimeZone};

const DATE_FORMAT: &str = "%e %b %Y %I:%M%P %Z";

pub fn run<Tz>(record: &Record<Tz>) -> Result<()>
where
    Tz: ContextTimeZone,
{
    println!("| {:<40} | {:<40} |", "Check-in", "Check-out");
    for _ in 0..89 {
        print!("=");
    }
    println!();
    for entry in record.get_entries() {
        let local_check_in_date = &entry.check_in;
        let local_check_out_date = entry.get_check_out()?;
        print!(
            "| {:<40} | {:<40} |",
            local_check_in_date.format(DATE_FORMAT),
            local_check_out_date.format(DATE_FORMAT),
        );
        println!();
    }

    if let Some((current_session, _)) = record.get_current_session() {
        println!(
            "| {:<40} | {:<40} |",
            current_session.format(DATE_FORMAT),
            "-",
        );
    }

    Ok(())
}
