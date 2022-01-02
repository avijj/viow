mod error;
mod wave;
mod formatting;
mod load;
mod scripts;
mod viewer;
mod data;
mod pipeline;

use error::*;
use wave::Wave;
use viewer::*;
use load::vcd::VcdLoader;
use scripts::{
    ScriptState, RunCommand,
    lua::LuaInterpreter
};
use data::{SimTime, SimTimeUnit};

use anyhow::Result;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::layout::{ Layout, Constraint, Direction };
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyEvent, KeyCode}
};
use clap::Parser;
use std::time::Duration;
use std::path::PathBuf;
use std::io;


fn render_loop(stdout: std::io::Stdout, opts: Opts) -> Result<()> {
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //let wave = Wave::new();
    //let loader = TestLoader::new(200, 2000);
    let opt_timeunits = opts.timeunits.trim().to_lowercase();
    let cycle_time = SimTime::new(opts.clock_period, SimTimeUnit::from_string(opt_timeunits)?);
    let loader = Box::new(VcdLoader::new(PathBuf::from(opts.input), cycle_time)?);
    let wave = Wave::load(loader)?;
    //let mut interpreter = LuaInterpreter::new(state, wave);
    let mut state = ScriptState {
        ui: State::new(), 
        wv: wave,
    };
    let mut interpreter = LuaInterpreter::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let stack = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1)
                ])
                .split(size);

            //let state = interpreter.state_mut();
            //let wave = interpreter.wave_mut();
            state.ui.resize(stack[0].width - 48, stack[0].height - 2);
            state.ui.data_size(state.wv.num_signals(), state.wv.num_cycles());
            
            let table = build_table(&state.wv, &state.ui);
            f.render_stateful_widget(table, size, state.ui.get_mut_table_state());

            let statusline = build_statusline(&state.ui);
            f.render_widget(statusline, stack[1]);

            let commandline = build_commandline(&state.ui);
            f.render_widget(commandline, stack[2]);
        })?;

        // check events
        if event::poll(Duration::from_millis(200))? {
            let ev = event::read()?;
            //let state = interpreter.state_mut();

            if state.ui.in_command() {
                match ev {
                    Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                        state.ui.put_command(c);
                    }

                    Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                        let cmd = state.ui.pop_command()
                            .ok_or(Error::NoCommand)?;
                        state = interpreter.run_command(state, cmd)?;
                    }

                    Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                        state.ui.take_command();
                    }

                    _ => {}
                }
            } else {
                match ev {
                    // quit
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                        break
                    }

                    // down
                    Event::Key(KeyEvent { code: KeyCode::Char('j'), .. }) => {
                        state.ui.move_cursor_down();
                    }

                    // page down
                    Event::Key(KeyEvent { code: KeyCode::Char('J'), .. }) => {
                        state.ui.move_page_down();
                    }

                    // up
                    Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => {
                        state.ui.move_cursor_up();
                    }
                    
                    // page up
                    Event::Key(KeyEvent { code: KeyCode::Char('K'), .. }) => {
                        state.ui.move_page_up();
                    }
                    
                    // left
                    Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => {
                        state.ui.move_cursor_left();
                    }

                    // page left
                    Event::Key(KeyEvent { code: KeyCode::Char('H'), .. }) => {
                        state.ui.move_page_left();
                    }

                    // right
                    Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => {
                        state.ui.move_cursor_right();
                    }

                    // page right
                    Event::Key(KeyEvent { code: KeyCode::Char('L'), .. }) => {
                        state.ui.move_page_right();
                    }

                    // zoom in '+'
                    Event::Key(KeyEvent { code: KeyCode::Char('+'), .. }) => {
                        state.ui.zoom_in();
                    }

                    // zoom out '-'
                    Event::Key(KeyEvent { code: KeyCode::Char('-'), .. }) => {
                        state.ui.zoom_out();
                    }

                    Event::Key(KeyEvent { code: KeyCode::Char(':'), .. }) => {
                        state.ui.start_command();
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}


/// Display a wave file in the console.
#[derive(Parser)]
struct Opts {
    /// Input file with data to display
    input: String,

    /// Clock period in number of timeunits to sample displayed data
    #[clap(short, long)]
    clock_period: u64,

    /// Timeunits to use to interpret times given in arguments
    #[clap(short, long)]
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
