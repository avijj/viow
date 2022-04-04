use super::*;
use crate::pipeline::filter;
use crate::load::plugin::PluggedLoader;

pub(super) fn load_vcd<'callback>(_lua: &'callback Lua, args: (String, u64, String)) -> mlua::Result<Wave>
{
    let (filename, period, timeunit) = args;
    let cycle_time = SimTime::new(period, SimTimeUnit::from_string(timeunit)?);
    let loader = Box::new(VcdLoader::new(PathBuf::from(filename), Some(cycle_time))?);
    let new_wave = Wave::load(loader)?;
    Ok(new_wave)
}

pub(super) fn load<'callback>(lua: &'callback Lua, args: (String, u64, String)) -> mlua::Result<Wave> {
    let plugins: Plugins = lua.globals().get("_plugins")?;
    let (filename, period, timeunit) = args;
    let suffix = filename.split('.').last()
        .ok_or(Error::UnknownFileFormat(filename.clone()))?;

    if suffix == "vcd" {
        load_vcd(lua, (filename, period, timeunit))
    } else {
        if let Some(plugin) = plugins.plugin_map.get(suffix) {
            let cycle_time = SimTime::new(period, SimTimeUnit::from_string(timeunit)?);
            let loader = Box::new(PluggedLoader::new(plugin.clone(), filename.as_str(), cycle_time)?);
            let new_wave = Wave::load(loader)?;
            Ok(new_wave)
        } else {
            Err(Error::UnknownFileFormat(filename.clone()).into())
        }
    }
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
