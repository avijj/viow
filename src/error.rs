use std::ops::Range;
use std::sync::Arc;
use mlua;
use regex;
use thiserror::Error;
use viow_plugin_api::error::Error as PluginError;
use abi_stable::library::LibraryError;
use rustyline::error::ReadlineError;


#[derive(Error,Debug)]
pub enum Error {
    #[error("Command line argument {0:} is required: {1:}")]
    MissingArgument(String, String),

    #[error("Do not know how to load '{0:}'")]
    UnknownFileFormat(String),

    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("Named signal '{0:}' not found")]
    NotFound(String),

    #[error("ID {0:} is out of range {} to {}", .1.start, .1.end)]
    IdOutOfRange(usize, std::ops::Range<usize>),

    #[error("Cycle {0:} is out of range 0 to {1:}")]
    CycleOutOfRange(usize, usize),

    #[error("The given range {0:?} is invalid within limits of {1:?}.")]
    InvalidRange(Range<usize>, Range<usize>),

    #[error("The given text '{0:}' can not be interpreted as time.")]
    InvalidTime(String),

    #[error("Regex error")]
    RegexErr(#[from] regex::Error),

    #[error("Error in Lua interpreter: {0:}")]
    LuaError(#[from] mlua::Error),

    #[error("No command specified")]
    NoCommand,

    #[error("Unexpected mode: {0:}")]
    WrongMode(String),

    #[error("Internal error: {0:}")]
    Internal(String),

    #[error("Error when loading plugin library (abi_stable): {0:}")]
    PluginLibrary(#[from] LibraryError),

    #[error("Error in plugin: {0:}")]
    Plugin(#[from] PluginError),

    #[error("Readline error: {0:}")]
    Readline(#[from] ReadlineError),
}

pub type Result<T> = std::result::Result<T, Error>;



impl From<Error> for mlua::Error {
    fn from(err: Error) -> Self {
        mlua::Error::ExternalError(Arc::new(err))
    }
}
