pub mod config;
pub mod data;
pub mod error;
pub mod formatting;
pub mod load;
pub mod pipeline;
pub mod scripts;
pub mod viewer;
pub mod wave;

use config::Config;
use data::{SimTime, SimTimeUnit};
use error::*;
use load::{empty::EmptyLoader, vcd::VcdLoader, plugin::PluggedLoader};
use scripts::{lua::LuaInterpreter, RunCommand, ScriptState};
use viewer::*;
use wave::Wave;
use formatting::WaveFormat;

//use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::path::PathBuf;
use std::rc::Rc;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;
use viow_plugin_api::{load_root_module_in_directory, FiletypeLoader_Ref};
use std::collections::HashMap;


type PluginMap = HashMap<String, FiletypeLoader_Ref>;

pub struct Step {
    pub state: ScriptState,
    pub interpreter: LuaInterpreter,
    pub should_exit: bool,
}

fn event_step_insert(
    ev: Event,
    step: Step,
) -> Result<Step> {
    let Step { mut state, interpreter, should_exit } = step;

    match ev {
        Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => {
            let new_name_list = state.ui.get_insert_list().unwrap().clone();
            state.wv.get_config_mut().name_list = new_name_list;
            state.wv.reconfigure()?;
            state.wv = state.wv.reload()?;
            state.ui.exit_insert_mode()?;
        }

        Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        }) => {
            state.ui.put_key(c);
        }

        Event::Key(KeyEvent {
            code: KeyCode::Enter,
            ..
        }) => {
            state.ui.enter_key();
            //if let Some(insertion) = state.ui.get_suggestion() {
            //state.wv.get_config_mut().name_list
            //.push(insertion.clone());
            //state.wv.reconfigure()?;
            //state.wv = state.wv.reload()?;
            //}
        }

        Event::Key(KeyEvent {
            code: KeyCode::Tab, ..
        }) => {
            state.ui.next_suggestion();
        }

        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => {
            state.ui.take_key();
        }

        _ => {}
    }

    Ok(Step { state, interpreter, should_exit })
}

fn event_step_normal(
    ev: Event,
    step: Step
) -> Result<Step> {
    let Step { mut state, mut interpreter, mut should_exit } = step;

    if state.ui.in_command() {
        match ev {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => {
                state.ui.put_command(c);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => {
                let cmd = state.ui.pop_command().ok_or(Error::NoCommand)?;
                state = interpreter.run_command(state, cmd)?;
            }

            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                state.ui.take_command();
            }

            _ => {}
        }
    } else {
        match ev {
            // quit
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }) => {
                should_exit = true;
            }

            // down
            Event::Key(KeyEvent {
                code: KeyCode::Char('j'),
                ..
            }) => {
                state.ui.move_cursor_down();
            }

            // page down
            Event::Key(KeyEvent {
                code: KeyCode::Char('J'),
                ..
            }) => {
                state.ui.move_page_down();
            }

            // up
            Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                ..
            }) => {
                state.ui.move_cursor_up();
            }

            // page up
            Event::Key(KeyEvent {
                code: KeyCode::Char('K'),
                ..
            }) => {
                state.ui.move_page_up();
            }

            // left
            Event::Key(KeyEvent {
                code: KeyCode::Char('h'),
                ..
            }) => {
                state.ui.move_cursor_left();
            }

            // page left
            Event::Key(KeyEvent {
                code: KeyCode::Char('H'),
                ..
            }) => {
                state.ui.move_page_left();
            }

            // right
            Event::Key(KeyEvent {
                code: KeyCode::Char('l'),
                ..
            }) => {
                state.ui.move_cursor_right();
            }

            // page right
            Event::Key(KeyEvent {
                code: KeyCode::Char('L'),
                ..
            }) => {
                state.ui.move_page_right();
            }

            // next transition
            Event::Key(KeyEvent { code: KeyCode::Char('w'), .. }) => {
                if state.ui.get_cursor_row().is_some() {
                    let cur_row = state.ui.get_cur_wave_row();
                    if let Some(next) = state.wv.cached_next_transition(cur_row, state.ui.get_cur_wave_col()) {
                        state.ui.set_cur_wave_col(next);
                    }
                }
            }

            // prev transition
            Event::Key(KeyEvent { code: KeyCode::Char('b'), .. }) => {
                if state.ui.get_cursor_row().is_some() {
                    let cur_row = state.ui.get_cur_wave_row();
                    if let Some(prev) = state.wv.cached_prev_transition(cur_row, state.ui.get_cur_wave_col()) {
                        state.ui.set_cur_wave_col(prev);
                    }
                }
            }

            // zoom in '+'
            Event::Key(KeyEvent {
                code: KeyCode::Char('+'),
                ..
            }) => {
                state.ui.zoom_in();
            }

            // zoom out '-'
            Event::Key(KeyEvent {
                code: KeyCode::Char('-'),
                ..
            }) => {
                state.ui.zoom_out();
            }

            // Enter command ':'
            Event::Key(KeyEvent {
                code: KeyCode::Char(':'),
                ..
            }) => {
                state.ui.start_command();
            }

            // Enter insert mode 'i'
            Event::Key(KeyEvent {
                code: KeyCode::Char('i'),
                ..
            }) => {
                state.wv.get_config_mut().enable_filter_list = false;
                state.wv.reconfigure()?;
                state.wv = state.wv.reload()?;
                let unfiltered = state.wv.get_names().clone();

                state.wv.get_config_mut().enable_filter_list = true;
                state.wv.reconfigure()?;
                state.wv = state.wv.reload()?;
                let insert_at = state.ui.get_cursor_row().unwrap_or(0);
                state
                    .ui
                    .start_insert_mode(unfiltered, state.wv.get_names().clone(), insert_at);
            }

            Event::Key(KeyEvent {
                code: KeyCode::Char('t'),
                ..
            }) => {
                if let Some(cur_row) = state.ui.get_cursor_row() {
                    let cur_fmt = state.wv.formatter(cur_row);
                    let next_fmt = match cur_fmt {
                        WaveFormat::Vector(sz) => WaveFormat::BitVector(sz),
                        WaveFormat::BitVector(sz) => WaveFormat::Vector(sz),

                        other => other
                    };

                    state.wv.set_formatter(cur_row, next_fmt);
                }
            }

            _ => {}
        }
    }

    Ok(Step { state, interpreter, should_exit })
}

pub fn event_step(step: Step, ev: Event) -> Result<Step> {
    if step.state.ui.in_insert_mode() {
        event_step_insert(ev, step)
    } else {
        event_step_normal(ev, step)
    }
}


#[cfg(not(tarpaulin_include))]
pub fn main_loop(stdout: std::io::Stdout, opts: Opts, config: Rc<Config>) -> Result<()> {
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut step = setup(opts, config)?;

    loop {
        step = render_step(&mut terminal, step)?;
        step = event_step(step, event::read()?)?;

        if step.should_exit {
            break;
        }
    }

    Ok(())
}

pub fn render_step(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    step: Step,
) -> Result<Step> {
    let Step { mut state, interpreter, should_exit } = step;

    terminal.draw(|f| {
        let size = f.size();
        let stack = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);

        state.ui.resize(
            stack[0].width.saturating_sub(48),
            stack[0].height.saturating_sub(2),
        );
        state
            .ui
            .data_size(state.wv.num_signals(), state.wv.num_cycles());

        if state.ui.in_insert_mode() {
            render_insert(f, &stack[0], &mut state.ui);
        } else {
            let (constraint, table) = build_table(&mut state.wv, &state.ui);
            let table = table.widths(&constraint);
            f.render_stateful_widget(table, size, state.ui.get_mut_table_state());
        }

        let statusline = build_statusline(&state.ui);
        f.render_widget(statusline, stack[1]);

        let commandline = build_commandline(&state.ui);
        f.render_widget(commandline, stack[2]);
    })?;

    Ok(Step {
        state,
        interpreter,
        should_exit
    })
}

fn load_plugins(opts: &Opts) -> Result<PluginMap> {
    let mut rv = PluginMap::new();

    for plugin_dir in opts.plugin.iter() {
        let plugin = load_root_module_in_directory(plugin_dir)?;
        let name = plugin.get_name()().to_string();
        let load_plugin = plugin.get_loader()()
            .into_option()
            .ok_or(Error::Internal(format!("Plugin '{name}' is not a file loader")))?;
        let suffix = load_plugin.get_suffix()().into_string();

        rv.insert(suffix, load_plugin);
    }

    Ok(rv)
}

pub fn setup(opts: Opts, config: Rc<Config>) -> Result<Step> {
    let plugins = load_plugins(&opts)?;

    if opts.input.ends_with(".vcd") {
        //let cycle_step = opts.cycle_step.ok_or(Error::MissingArgument(
            //"--clock-period".into(),
            //"Required to load a vcd file".into(),
        //))?;
        let timeunits = SimTimeUnit::from_string(opts.timeunits.trim().to_lowercase())?;
        let cycle_time = opts.cycle_step
            .map(|cs| SimTime::new(cs, timeunits));
        let loader = Box::new(VcdLoader::new(PathBuf::from(opts.input), cycle_time)?);
        let wave = Wave::load(loader/*, &config*/)?;

        //let mut interpreter = LuaInterpreter::new(state, wave);
        let state = ScriptState {
            ui: State::new(),
            wv: wave,
        };
        let interpreter = LuaInterpreter::new(&config)?;

        let step = Step { state, interpreter, should_exit: false };
        Ok(step)
    } else if opts.input.ends_with(".lua") {
        let loader = Box::new(EmptyLoader::new());
        let wave = Wave::load(loader/*, &config*/)?;

        let state = ScriptState {
            ui: State::new(),
            wv: wave,
        };
        let mut interpreter = LuaInterpreter::new(&config)?;
        let state = interpreter.run_file(state, opts.input)?;

        let step = Step { state, interpreter, should_exit: false };
        Ok(step)
    } else if !plugins.is_empty() {
        let suffix = opts.input.split('.').last()
            .ok_or(Error::UnknownFileFormat(opts.input.clone()))?;

        if let Some(plugin) = plugins.get(suffix) {
            let timeunits = SimTimeUnit::from_string(opts.timeunits.trim().to_lowercase())?;
            let cycle_time = opts.cycle_step
                .map(|cs| SimTime::new(cs, timeunits))
                .ok_or(Error::MissingArgument("cycle_step".into(), "Needed for plugin load".into()))?;
            let loader = Box::new(PluggedLoader::new(plugin.clone(), opts.input.as_str(), cycle_time)?);
            let wave = Wave::load(loader/*, &config*/)?;

            let state = ScriptState {
                ui: State::new(),
                wv: wave,
            };
            let interpreter = LuaInterpreter::new(&config)?;
            let step = Step { state, interpreter, should_exit: false };
            Ok(step)
        } else {
            Err(Error::UnknownFileFormat(opts.input.clone()))
        }
    } else {
        Err(Error::UnknownFileFormat(opts.input.clone()))
    }
}

/// Display a wave file in the console.
#[derive(Parser)]
pub struct Opts {
    /// Input file with data to display
    input: String,

    /// Determines how long one cycle is in timeunits
    #[clap(short, long)]
    cycle_step: Option<u64>,

    /// Timeunits to use to interpret times given in arguments
    #[clap(short, long, default_value = "ps")]
    timeunits: String,

    /// Load plugin on startup
    #[clap(long)]
    plugin: Vec<std::path::PathBuf>,
}
