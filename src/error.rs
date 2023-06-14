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

    #[error("There was an attempt to create an entry with check-out before check-in.")]
    CheckOutBeforeCheckIn,

    #[error("An entry has an invalid number of tokens")]
    EntryIncorrectNumberOfTokens,

    #[error("Not currently clocked-in.")]
    NotClockedIn,

    #[error("Already clocked-in.")]
    AlreadyClockedIn,

    #[error("Failed to parse an integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Failed to convert an integer: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),

    #[error("An entry was so long that it overflowed")]
    DateTimeOverflow,

    #[error("Formatting error: {0}")]
    Format(#[from] std::fmt::Error),

    #[error("Please set the path of your default editor using the $EDITOR environment variable")]
    MissingEditorPath,

    #[error("The editor subprocess exited unsuccessfully.")]
    UnsuccessfulEditor,
}
