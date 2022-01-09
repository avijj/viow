mod data;
mod error;
mod formatting;
mod load;
mod pipeline;
mod scripts;
mod viewer;
mod wave;

use data::{SimTime, SimTimeUnit, QuerySource};
use error::*;
use load::{empty::EmptyLoader, vcd::VcdLoader};
use scripts::{lua::LuaInterpreter, RunCommand, ScriptState};
use viewer::*;
use wave::Wave;

//use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Terminal;

fn event_loop_insert(
    ev: Event,
    mut state: ScriptState,
    interpreter: LuaInterpreter,
) -> Result<((ScriptState, LuaInterpreter), bool)> {
    match ev {
        Event::Key(KeyEvent {
            code: KeyCode::Esc,
            ..
        }) => {
            state.ui.exit_insert_mode();
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
        }

        Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            ..
        }) => {
            state.ui.take_key();
        }

        _ => {}
    }

    Ok(((state, interpreter), false))
}

fn event_loop_normal(
    ev: Event,
    mut state: ScriptState,
    mut interpreter: LuaInterpreter,
) -> Result<((ScriptState, LuaInterpreter), bool)> {
    let mut should_exit = false;

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
                state.ui.start_insert_mode(unfiltered);
            }

            _ => {}
        }
    }

    Ok(((state, interpreter), should_exit))
}

fn event_loop(
    state: ScriptState,
    interpreter: LuaInterpreter,
) -> Result<((ScriptState, LuaInterpreter), bool)> {
    if event::poll(Duration::from_millis(200))? {
        let ev = event::read()?;
        //let state = interpreter.state_mut();

        if state.ui.in_insert_mode() {
            event_loop_insert(ev, state, interpreter)
        } else {
            event_loop_normal(ev, state, interpreter)
        }
    } else {
        Ok(((state, interpreter), false))
    }
}

fn render_loop(stdout: std::io::Stdout, opts: Opts) -> Result<()> {
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let (mut state, mut interpreter) = setup(opts)?;
    loop {
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

            state.ui.resize(stack[0].width - 48, stack[0].height - 2);
            state
                .ui
                .data_size(state.wv.num_signals(), state.wv.num_cycles());

            if state.ui.in_insert_mode() {
                let substack = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([
                        Constraint::Ratio(1, 3),
                        Constraint::Length(1),
                        Constraint::Ratio(2, 3),
                    ])
                    .split(stack[0]);

                let (list, suggestions, prompt) = build_insert(&state.wv, &state.ui);
                f.render_widget(list, substack[0]);
                f.render_widget(prompt, substack[1]);
                f.render_widget(suggestions, substack[2]);
            } else {
                let table = build_table(&state.wv, &state.ui);
                f.render_stateful_widget(table, size, state.ui.get_mut_table_state());
            }

            let statusline = build_statusline(&state.ui);
            f.render_widget(statusline, stack[1]);

            let commandline = build_commandline(&state.ui);
            f.render_widget(commandline, stack[2]);
        })?;

        // check events
        let ((new_state, new_interpreter), should_exit) = event_loop(state, interpreter)?;
        state = new_state;
        interpreter = new_interpreter;

        if should_exit {
            break;
        }
    }

    Ok(())
}

fn setup(opts: Opts) -> Result<(ScriptState, LuaInterpreter)> {
    if opts.input.ends_with(".vcd") {
        let clock_period = opts.clock_period.ok_or(Error::MissingArgument(
            "--clock_period".into(),
            "Required to load a vcd file".into(),
        ))?;
        let opt_timeunits = opts.timeunits.trim().to_lowercase();
        let cycle_time = SimTime::new(clock_period, SimTimeUnit::from_string(opt_timeunits)?);
        let loader = Box::new(VcdLoader::new(PathBuf::from(opts.input), cycle_time)?);
        let wave = Wave::load(loader)?;

        //let mut interpreter = LuaInterpreter::new(state, wave);
        let state = ScriptState {
            ui: State::new(),
            wv: wave,
        };
        let interpreter = LuaInterpreter::new()?;

        Ok((state, interpreter))
    } else if opts.input.ends_with(".lua") {
        let loader = Box::new(EmptyLoader::new());
        let wave = Wave::load(loader)?;

        let state = ScriptState {
            ui: State::new(),
            wv: wave,
        };
        let mut interpreter = LuaInterpreter::new()?;
        let state = interpreter.run_file(state, opts.input)?;

        Ok((state, interpreter))
    } else {
        Err(Error::UnknownFileFormat(opts.input.clone()))
    }
}

/// Display a wave file in the console.
#[derive(Parser)]
struct Opts {
    /// Input file with data to display
    input: String,

    /// Clock period in number of timeunits to sample displayed data
    #[clap(short, long)]
    clock_period: Option<u64>,

    /// Timeunits to use to interpret times given in arguments
    #[clap(short, long, default_value = "ps")]
    timeunits: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut stdout = io::stdout();

    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    match render_loop(stdout, opts) {
        Ok(_) => {
            crossterm::terminal::disable_raw_mode()?;
            let mut stdout = io::stdout();
            stdout.execute(LeaveAlternateScreen)?;
            Ok(())
        }

        Err(err) => {
            crossterm::terminal::disable_raw_mode()?;
            let mut stdout = io::stdout();
            stdout.execute(LeaveAlternateScreen)?;

            println!("Error: {}", err);
            Err(err)
        }
    }
}
