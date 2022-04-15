mod api;

use crate::PluginMap;
use super::*;
use crate::config::Config;
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
use std::path::{Path,PathBuf};
use std::stringify;

macro_rules! add_global_function {
    ($lua:expr, $name:ident) => {
        $lua.globals().set(stringify!($name), $lua.create_function(api::$name)?)?;
    }
}

pub struct LuaInterpreter {
    lua: Lua,
}

impl LuaInterpreter {
    pub fn new(config: impl AsRef<Config>, plugin_map: PluginMap) -> Result<Self> {
        let lua = Lua::new();

        Self::install_plugins(&lua, plugin_map)?;
        Self::configure_lua_path(&lua, config)?;

        let work_dir = std::env::current_dir()?;
        Self::set_working_directory(&lua, &work_dir)?;

        add_global_function!(lua, open);
        add_global_function!(lua, filter_signals);
        add_global_function!(lua, grep);
        add_global_function!(lua, ignore);
        add_global_function!(lua, allow_deny);
        add_global_function!(lua, remove_comments);
        add_global_function!(lua, pop_filter);
        add_global_function!(lua, replace_prefix);
        add_global_function!(lua, analog);

        // Try to load viow.lua as entry to standard library. Silently ignore if not found.
        let chunk = lua.load("require('viow')")
            .set_name("init")?;
        let _ = chunk.exec();

        Ok(Self {
            lua
        })
    }

    fn install_plugins(lua: &Lua, plugin_map: PluginMap) -> Result<()> {
        let plugins = Plugins { plugin_map };
        lua.globals().set("_plugins", plugins)?;
        Ok(())
    }

    fn set_working_directory(lua: &Lua, work_dir: impl AsRef<Path>) -> Result<()> {
        let wd_str: String = work_dir.as_ref()
            .to_str()
            .ok_or(Error::Internal(format!("Can not represent '{}' as string for Lua",
                        work_dir.as_ref().display())))?
            .into();
        lua.globals().set("_cwd", wd_str)?;
        Ok(())
    }

    fn configure_lua_path(lua: &Lua, config: impl AsRef<Config>) -> Result<()> {
        let lua_path = config.as_ref().get_script_dir()
            .and_then(|script_path| script_path.to_str())
            .map(|dirname| format!("?.lua;{0:}/?.lua", dirname));

        if let Some(lua_path) = lua_path {
            let package: mlua::Table = lua.globals().get("package")?;
            let path: String = package.get("path")?;
            let new_path = format!("{};{}", path, lua_path);
            package.set("path", new_path)?;
        }

        Ok(())
    }

    pub fn run_file(&mut self, state: ScriptState, filename: impl AsRef<str>) -> Result<ScriptState> {
        // set working directory
        let path = PathBuf::from(filename.as_ref());
        let dir = path.canonicalize()?
            .parent()
            .ok_or(Error::NotFound(format!("'{}': can not determine parent directory",
                        filename.as_ref())))?
            .to_path_buf();
        Self::set_working_directory(&self.lua, dir)?;

        // load and run file
        let mut file = File::open(filename.as_ref())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        self.run_command(state, contents)
    }
}


#[derive(Clone)]
struct View {
    cursor_col: usize,
    cursor_row: Option<usize>,
}

impl View {
    fn new(state: &viewer::State) -> Self {
        Self {
            cursor_col: state.get_cur_wave_col(),
            cursor_row: state.get_cursor_row(),
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


#[derive(Clone)]
struct Plugins {
    plugin_map: PluginMap,
}

impl UserData for Plugins { }


impl RunCommand for LuaInterpreter {
    fn run_command(&mut self, mut state: ScriptState, command: String) -> Result<ScriptState> {
        self.lua.globals().set("view", View::new(&state.ui))?;
        self.lua.globals().set("wave", state.wv)?;

        let chunk = self.lua.load(&command)
            .set_name("Command")?;

        if let Err(script_error) = chunk.exec() {
            state.er = Some(script_error.into());
        }

        let view: View = self.lua.globals().get("view")?;
        view.update_state(&mut state.ui);

        let new_wave: Wave = self.lua.globals().get("wave")?;
        self.lua.globals().raw_remove("wave")?;
        state.wv = new_wave;

        Ok(state)
    }
}
