use super::*;
use crate::viewer;
use crate::data::*;
use crate::wave::*;
use crate::load::vcd::VcdLoader;
use mlua::{
    self as lua,
    Lua,Value,FromLua,UserData,UserDataFields
};

use std::path::PathBuf;

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

impl UserData for Wave { }

impl<'lua> FromLua<'lua> for Wave {
    fn from_lua(lua_value: Value<'lua>, _: &'lua Lua) -> lua::Result<Self> {
        match lua_value {
            Value::UserData(data) => {
                let rv: Self = data.take()?;
                Ok(rv)
            }

            _ => {
                Err(lua::Error::FromLuaConversionError { from: "userdata", to: "Wave", message: None })
            }
        }
    }
}

impl RunCommand for LuaInterpreter {
    fn run_command(&mut self, mut state: ScriptState, command: String) -> Result<ScriptState> {
        //let mut new_wave = None;

        let load = self.lua.create_function_mut(|_, args: (String, u64, String)| {
            let (filename, period, timeunit) = args;
            let cycle_time = SimTime::new(period, SimTimeUnit::from_string(timeunit)?);
            let loader = Box::new(VcdLoader::new(PathBuf::from(filename), cycle_time)?);
            let new_wave = Wave::load(loader)?;
            Ok(new_wave)
        })?;

        self.lua.globals().set("view", View::new(&state.ui))?;
        self.lua.globals().set("wave", state.wv)?;
        self.lua.globals().set("load", load)?;

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
