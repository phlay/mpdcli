use std::{io, fmt};
use futures_channel::mpsc;
use mpd_client::{
    protocol::MpdProtocolError,
    client::CommandError,
};

#[derive(Clone, Debug)]
pub enum Error {
    Io(io::ErrorKind),
    Mpd(String),
    InvalidQueue,
    SendError(mpsc::SendError),
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Mpd(msg) => write!(f, "mpd error: {msg}"),
            Self::InvalidQueue => write!(f, "queue error in mpd"),
            Self::SendError(error) => write!(f, "send to channel: {error}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

impl From<MpdProtocolError> for Error {
    fn from(error: MpdProtocolError) -> Self {
        match error {
            MpdProtocolError::Io(ioerr) => Self::Io(ioerr.kind()),
            _ => Self::Mpd(error.to_string()),
        }
    }
}

impl From<CommandError> for Error {
    fn from(error: CommandError) -> Self {
        Self::Mpd(error.to_string())
    }
}

impl From<mpsc::SendError> for Error {
    fn from(error: mpsc::SendError) -> Self {
        Self::SendError(error)
    }
}
