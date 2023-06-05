use std::fmt::Display;

use chrono::{DateTime, Duration, Local};

#[derive(Debug)]
pub struct Ago(pub chrono::Duration);

impl Display for Ago {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let days = self.0.num_days().abs();
        let hours = self.0.num_hours().abs();
        let minutes = self.0.num_minutes().abs();

        if days > 0 {
            write!(
                f,
                "{days} days, {} hours, {} minutes",
                hours % 24,
                minutes % 60
            )?;
        } else if hours > 0 {
            write!(f, "{} hours, {} minutes", hours % 24, minutes % 60)?;
        } else {
            write!(f, "{} minutes", minutes % 60)?;
        }

        if self.0 < Duration::zero() {
            write!(f, " ago")
        } else {
            write!(f, " from now")
        }
    }
}

pub type Result<T, E = Main> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Main {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Date parsing error: {0}")]
    ChronoParse(#[from] chrono::ParseError),

    #[error("Punch Clock is already initialized.")]
    AlreadyInitialized,

    #[error("Punch Clock is not initialized")]
    Uninitialized,

    #[error("Record has an invalid entry")]
    InvalidEntry,

    #[error("Invalid event string: {0}")]
    InvalidEventString(String),

    #[error("The previous entry is in the future! ({0}, {1})")]
    LastEntryInFuture(DateTime<Local>, Ago),

    #[error("Already checked in since {0} ({1})")]
    AlreadyCheckedIn(DateTime<Local>, Ago),

    #[error("Already checked out since {0} ({1})")]
    AlreadyCheckedOut(DateTime<Local>, Ago),

    #[error("First entry must be 'enter'")]
    ExitingFirst,
}
