use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    InvalidInput,
    UnknownCommand,
    InvalidUuid,
    MissingArgument,
    InvalidUserKind,
    MissingUserName,
    MissingCommandName,
    IO(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput => write!(f, "invalid or empty input"),
            Self::InvalidUuid => write!(f, "invalid uuid"),
            Self::MissingArgument => write!(f, "missing argument"),
            Self::UnknownCommand => write!(f, "unknown command"),
            Self::InvalidUserKind => write!(f, "invalid user kind"),
            Self::MissingUserName => write!(f, "missing user name"),
            Self::MissingCommandName => write!(f, "missing command name"),
            Self::IO(e) => write!(f, "input/output error: {e}"),
        }
    }
}
