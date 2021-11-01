use super::*;
use crate::viewer;
use mlua::{Lua,UserData,UserDataFields};

pub struct LuaInterpreter {
    lua: Lua,
}

impl LuaInterpreter {
    pub fn new() -> Self {
        Self {
            lua: Lua::new()
        }
    }
    
}


#[derive(Clone)]
struct View {
    cursor_col: usize,
    cursor_row: usize,
}

impl View {
    fn new(state: &viewer::State) -> Self {
        Self {
            cursor_col: state.get_cur_wave_col(),
            cursor_row: state.get_cur_wave_row(),
        }
    }

    fn update_state(&self, state: &mut viewer::State) {
        state.set_cur_wave_col(self.cursor_col);
        state.set_cur_wave_row(self.cursor_row);
    }
}

impl UserData for View {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("cursor_col", |_, this| Ok(this.cursor_col));
        fields.add_field_method_set("cursor_col", |_, this, x| {
                this.cursor_col = x;
                Ok(())
            });

        fields.add_field_method_get("cursor_row", |_, this| Ok(this.cursor_row));
        fields.add_field_method_set("cursor_row", |_, this, x| {
                this.cursor_row = x;
                Ok(())
            });
    }
}

impl RunCommand for LuaInterpreter {
    fn run_command(&mut self, state: &mut viewer::State, command: String) -> Result<()> {
        self.lua.globals()
            .set("view", View::new(state))?;

        let chunk = self.lua.load(&command)
            .set_name("Command")?;
        chunk.exec()?;

        let view: View = self.lua.globals()
            .get("view")?;
        view.update_state(state);

        Ok(())
    }
}
