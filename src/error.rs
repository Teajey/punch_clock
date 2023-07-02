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

    #[error("Failed to parse os string as utf-8: {0:?}")]
    OsStringParseFail(std::ffi::OsString),

    #[error("Did not find file in path by prefix: {0}")]
    NoPrefixInPath(String),

    #[error("Asked for an out-of-bounds timezone offset: {0}")]
    TimezoneOutOfRange(i32),

    #[error("Date out of range")]
    DateOutOfRange,

    #[error("Start must be before end")]
    RangeStartPosition,
}
