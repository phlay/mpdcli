use std::{io, fmt};
use futures_channel::mpsc;
use mpd_client::{
    protocol::MpdProtocolError,
    client::CommandError,
    client::ConnectionError,
};

#[derive(Clone, Debug)]
pub enum Error {
    Io(io::ErrorKind),
    Mpd(String),
    MpdErrorResponse(u64),
    InvalidQueue,
    SendError(mpsc::SendError),
    Disconnect,
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Mpd(msg) => write!(f, "mpd error: {msg}"),
            Self::MpdErrorResponse(code) => write!(f, "mpd returned error code {code}"),
            Self::InvalidQueue => write!(f, "queue error in mpd"),
            Self::SendError(error) => write!(f, "send to channel: {error}"),
            Self::Disconnect => write!(f, "connection to mpd was disconnected"),
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
        use mpd_client::protocol::response::Error as MpdErr;
        match error {
            CommandError::ErrorResponse { error: MpdErr { code, .. }, .. }
                => Self::MpdErrorResponse(code),

            _ => Self::Mpd(error.to_string()),
        }
    }
}

impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Self::Mpd(error.to_string())
    }
}

impl From<mpsc::SendError> for Error {
    fn from(error: mpsc::SendError) -> Self {
        Self::SendError(error)
    }
}
