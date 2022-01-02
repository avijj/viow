pub mod lua;

use crate::error::*;
use crate::viewer;
use crate::wave::Wave;


pub struct ScriptState {
    pub ui: viewer::State,
    pub wv: Wave,
}

pub trait RunCommand {
    fn run_command(&mut self, state: ScriptState, command: String) -> Result<ScriptState>;
}
