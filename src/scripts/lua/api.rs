use super::*;
use crate::pipeline::filter;

pub(super) fn load_wave<'callback>(_lua: &'callback Lua, args: (String, u64, String)) -> mlua::Result<Wave>
{
    let (filename, period, timeunit) = args;
    let cycle_time = SimTime::new(period, SimTimeUnit::from_string(timeunit)?);
    let loader = Box::new(VcdLoader::new(PathBuf::from(filename), cycle_time)?);
    let new_wave = Wave::load(loader)?;
    Ok(new_wave)
}

pub(super) fn filter_signals<'callback>(_lua: &'callback Lua, args: (Wave, Vec<String>)) -> mlua::Result<Wave>
{
    let (wave, signals) = args;

    let filter = Box::new(filter::SignalList::new(signals));
    let wave = wave.push_filter(filter)?;

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

pub(super) fn pop<'callback>(_lua: &'callback Lua, wave: Wave) -> mlua::Result<Wave> {
    let (wave, _) = wave.pop_filter()?;
    Ok(wave)
}
