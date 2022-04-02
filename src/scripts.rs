pub mod lua;

use crate::error::*;
use crate::viewer;
use crate::wave::Wave;


pub struct ScriptState {
    pub ui: viewer::State,
    pub wv: Wave,
    pub er: Option<Error>,
}

impl std::fmt::Debug for ScriptState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.write_str("ScriptState {{ ... }}")
    }
}

pub trait RunCommand {
    fn run_command(&mut self, state: ScriptState, command: String) -> Result<ScriptState>;
}
