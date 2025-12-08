/*!
Provides this crate's [`Error`] and [`Result`] types as well as helper functions.

 */

use flat_error::FlatError;
use std::{
    error::Error as StdError,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{Error as IoError, ErrorKind as IoErrorKind},
    num::{ParseFloatError, ParseIntError},
    str::ParseBoolError,
};
use tracing::subscriber::SetGlobalDefaultError;

#[cfg(feature = "command-line")]
use tracing_subscriber::filter::ParseError;

// ------------------------------------------------------------------------------------------------
// Public Types
// ------------------------------------------------------------------------------------------------

///
/// The `Error` type for this crate.
///
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// An error was signaled by the standard library I/O functions.
    IoError {
        source: FlatError,
    },
    #[cfg(feature = "command-line")]
    EnvFilterError {
        source: FlatError,
    },
    SetGlobalError {
        source: FlatError,
    },
    /// Error parsing Rustfile
    SourceParserError,
    ValueParserError {
        type_name: &'static str,
        source: FlatError,
    },
    /// Multiple errors were aggregated from some function below.
    MultipleErrors {
        sources: Vec<Error>,
    },
    /// An unknown error occurred.
    Unknown {
        message: String,
    },
}

// ------------------------------------------------------------------------------------------------
// Public Functions
// ------------------------------------------------------------------------------------------------

/// Construct an `Error` from the provided source error.
#[inline]
pub fn io_error(source: IoError) -> Error {
    Error::from(source)
}

pub fn value_parser_error(type_name: &'static str, source: FlatError) -> Error {
    Error::ValueParserError { type_name, source }
}

// ------------------------------------------------------------------------------------------------
// Implementations
// ------------------------------------------------------------------------------------------------

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}",
            match self {
                Self::IoError { source } => format!("An I/O error occurred; source: {source}"),
                #[cfg(feature = "command-line")]
                Self::EnvFilterError { source } =>
                    format!("An error occurred parsing a tracing env-filter; source: {source}"),
                Self::SetGlobalError { source } => format!(
                    "An error occurred setting the global tracing subscriber; source:{source}"
                ),
                Self::SourceParserError =>
                    "An error occured parsing a Rustfile, reported by the parser or lexer previously.".to_string(),
                Self::ValueParserError { type_name, source } => format!(
                    "An error occured parsing a value of type `{type_name}`; source: {source}"
                ),
                Self::MultipleErrors { sources } => {
                    format!(
                        "Multiple errors occurred:\n{}",
                        sources
                            .iter()
                            .enumerate()
                            .map(|(i, e)| format!("{i:<3}. {e}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                }
                Self::Unknown { message } =>
                    format!("An unknown error occurred; message: {message}"),
            }
        )
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::IoError { source } => Some(source),
            #[cfg(feature = "command-line")]
            Self::EnvFilterError { source } => Some(source),
            Self::SetGlobalError { source } => Some(source),
            _ => None,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Implementations From
// ------------------------------------------------------------------------------------------------

impl From<IoError> for Error {
    fn from(source: IoError) -> Self {
        Self::IoError {
            source: FlatError::from_any(&source),
        }
    }
}

impl From<IoErrorKind> for Error {
    fn from(source: IoErrorKind) -> Self {
        let error: IoError = source.into();
        Self::IoError {
            source: FlatError::from_any(&error),
        }
    }
}

impl From<SetGlobalDefaultError> for Error {
    fn from(source: SetGlobalDefaultError) -> Self {
        Self::SetGlobalError {
            source: FlatError::from_any(&source),
        }
    }
}

#[cfg(feature = "command-line")]
impl From<ParseError> for Error {
    fn from(source: ParseError) -> Self {
        Self::EnvFilterError {
            source: FlatError::from_any(&source),
        }
    }
}

impl From<ParseBoolError> for Error {
    fn from(error: ParseBoolError) -> Self {
        Self::ValueParserError {
            type_name: "bool",
            source: FlatError::from_any(&error),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Self::ValueParserError {
            type_name: "int",
            source: FlatError::from_any(&error),
        }
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Self::ValueParserError {
            type_name: "float",
            source: FlatError::from_any(&error),
        }
    }
}

impl From<Vec<Error>> for Error {
    fn from(sources: Vec<Error>) -> Self {
        Self::MultipleErrors { sources }
    }
}

impl From<&[Error]> for Error {
    fn from(sources: &[Error]) -> Self {
        Self::MultipleErrors {
            sources: sources.to_vec(),
        }
    }
}

impl FromIterator<Error> for Error {
    fn from_iter<I: IntoIterator<Item = Error>>(iter: I) -> Self {
        Self::MultipleErrors {
            sources: iter.into_iter().collect(),
        }
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Unknown { message }
    }
}
