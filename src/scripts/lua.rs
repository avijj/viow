mod api;

use super::*;
use crate::viewer;
use crate::data::*;
use crate::wave::*;
use crate::load::vcd::VcdLoader;
use mlua::{
    self as lua,
    Lua,Value,FromLua,UserData,UserDataFields
};
use std::fs::File;

use std::io::Read;
use std::path::PathBuf;

pub struct LuaInterpreter {
    lua: Lua,
}

impl LuaInterpreter {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        let load = lua.create_function(api::load_wave)?;
        lua.globals().set("load_vcd", load)?;

        let filter_signals = lua.create_function(api::filter_signals)?;
        lua.globals().set("filter_signals", filter_signals)?;

        let grep = lua.create_function(api::grep)?;
        lua.globals().set("grep", grep)?;

        Ok(Self {
            lua
        })
    }

    pub fn run_file(&mut self, state: ScriptState, filename: impl AsRef<str>) -> Result<ScriptState> {
        let mut file = File::open(filename.as_ref())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.run_command(state, contents)
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

impl UserData for Wave { }

impl<'lua> FromLua<'lua> for Wave {
    fn from_lua(lua_value: Value<'lua>, _: &'lua Lua) -> lua::Result<Self> {
        match lua_value {
            Value::UserData(data) => {
                let rv: Self = data.take()?;
                Ok(rv)
            }

            Value::Error(err) => {
                Err(err)
            }

            Value::Nil => {
                Err(lua::Error::FromLuaConversionError { 
                    from: "userdata",
                    to: "Wave",
                    message: Some("Expected value of type Wave, but found Nil".to_string())
                })
            }

            _ => {
                Err(lua::Error::FromLuaConversionError { from: "userdata", to: "Wave", message: None })
            }
        }
    }
}

impl RunCommand for LuaInterpreter {
    fn run_command(&mut self, mut state: ScriptState, command: String) -> Result<ScriptState> {
        self.lua.globals().set("view", View::new(&state.ui))?;
        self.lua.globals().set("wave", state.wv)?;

        let chunk = self.lua.load(&command)
            .set_name("Command")?;
        chunk.exec()?;

        let view: View = self.lua.globals().get("view")?;
        view.update_state(&mut state.ui);

        let new_wave: Wave = self.lua.globals().get("wave")?;
        self.lua.globals().raw_remove("wave")?;
        state.wv = new_wave;

        Ok(state)
    }
}
