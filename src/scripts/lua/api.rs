use super::*;
use crate::pipeline::{SrcBox, filter};
use crate::load::plugin::PluggedLoader;

pub(super) fn open<'callback>(lua: &'callback Lua, args: (String, u64, String)) -> mlua::Result<Wave> {
    let (filename, period, timeunit) = args;

    let plugins: Plugins = lua.globals().get("_plugins")?;
    let work_dir: String = lua.globals().get("_cwd")?;

    let mut path = PathBuf::from(work_dir);
    path.push(filename);
    let suffix = path.extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .ok_or(Error::UnknownFileFormat(path.to_string_lossy().to_string()))?;
    //let suffix = filename.split('.').last()
        //.ok_or(Error::UnknownFileFormat(filename.clone()))?;

    let cycle_time = SimTime::new(period, SimTimeUnit::from_string(timeunit)?);
    let loader: SrcBox;

    if suffix == "vcd" {
        //load_vcd(lua, (path, period, timeunit))
        loader = Box::new(VcdLoader::new(path, Some(cycle_time))?);
    } else {
        if let Some(plugin) = plugins.plugin_map.get(&suffix) {
            let path_str = path.to_string_lossy();
            loader = Box::new(PluggedLoader::new(plugin.clone(), path_str, cycle_time)?);
        } else {
            return Err(Error::UnknownFileFormat(path.to_string_lossy().to_string()).into());
        }
    }

    let new_wave = Wave::load(loader)?;
    Ok(new_wave)
}

pub(super) fn filter_signals<'callback>(_lua: &'callback Lua, args: (Wave, Vec<String>)) -> mlua::Result<Wave>
{
    let (mut wave, signals) = args;

    wave.get_config_mut().name_list = signals.clone();
    let filter = Box::new(filter::SignalList::new(signals));
    let mut wave = wave.push_filter(filter)?;
    wave.reconfigure()?;

    Ok(wave)
}

pub(super) fn grep<'callback>(_lua: &'callback Lua, args: (Wave, String)) -> mlua::Result<Wave>
{
    let (wave, expr) = args;

    let filter = Box::new(filter::Grep::new(&expr)?);
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}

pub(super) fn ignore<'callback>(_lua: &'callback Lua, args: (Wave, Vec<String>)) -> mlua::Result<Wave>
{
    let (wave, deny_list) = args;

    let filter = Box::new(filter::Ignore::new(&vec![], &deny_list)?);
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}

pub(super) fn allow_deny<'callback>(_lua: &'callback Lua, args: (Wave, Vec<String>, Vec<String>)) -> mlua::Result<Wave>
{
    let (wave, allow_list, deny_list) = args;

    let filter = Box::new(filter::Ignore::new(&allow_list, &deny_list)?);
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}

pub(super) fn remove_comments<'callback>(_lua: &'callback Lua, wave: Wave) -> mlua::Result<Wave>
{
    let filter = Box::new(filter::RemoveComments::new());
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}

pub(super) fn pop_filter<'callback>(_lua: &'callback Lua, wave: Wave) -> mlua::Result<Wave> {
    let (wave, _) = wave.pop_filter()?;
    Ok(wave)
}

pub(super) fn replace_prefix<'callback>(_lua: &'callback Lua, args: (Wave, String, String)) -> mlua::Result<Wave>
{
    let (wave, from, to) = args;
    let filter = Box::new(filter::ReplacePrefix::new(from, to));
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}


pub(super) fn analog<'callback>(_lua: &'callback Lua, args: (Wave, Vec<String>, f64, f64)) -> mlua::Result<Wave>
{
    let (wave, patterns, min, max) = args;

    let filter = Box::new(filter::Analog::new(&patterns, min, max)?);
    let wave = wave.push_filter(filter)?;

    Ok(wave)
}
