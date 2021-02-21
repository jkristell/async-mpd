use std::io;
use std::num::ParseIntError;

/// Error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Server failed to parse the command
    #[error("Invalid command or arguments")]
    CommandError { msg: String },

    /// The server closed the connection
    #[error("The server closed the connection")]
    Disconnected,

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] io::Error),

    /// TODO
    #[error("Server error")]
    ServerError { msg: String },

    /// Generic unexpected response error
    #[error("invalid value error")]
    ValueError { msg: String },

    /// Conversion error
    #[error(transparent)]
    ParseInteError(#[from] ParseIntError),
}
