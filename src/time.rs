use std::fmt::{self, Display, Write};

use chrono::Duration;

pub fn human_readable_duration(duration: &Duration) -> Result<String, fmt::Error> {
    let mut buf = String::new();

    let days = duration.num_days().abs();
    let hours = duration.num_hours().abs();
    let minutes = duration.num_minutes().abs();

    if days > 0 {
        write!(
            buf,
            "{days} days, {} hours, {} minutes",
            hours % 24,
            minutes % 60
        )?;
    } else if hours > 0 {
        write!(buf, "{} hours, {} minutes", hours % 24, minutes % 60)?;
    } else {
        write!(buf, "{} minutes", minutes % 60)?;
    }

    Ok(buf)
}

#[derive(Debug)]
pub struct Ago(pub chrono::Duration);

impl Display for Ago {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", human_readable_duration(&self.0)?)?;
        if self.0 < Duration::zero() {
            write!(f, " ago")
        } else {
            write!(f, " from now")
        }
    }
}
