pub mod lua;

use crate::viewer;

use mlua;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum Error {
    #[error("Error in Lua interpreter")]
    LuaError(#[from] mlua::Error),

    #[error("No command specified")]
    NoCommand,
}

pub type Result<T> = std::result::Result<T, Error>;


pub trait RunCommand {
    fn run_command(&mut self, state: &mut viewer::State, command: String) -> Result<()>;
}
