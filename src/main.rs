mod error;
mod wave;
mod formatting;
mod load;
mod scripts;
mod viewer;
mod data;
mod pipeline;

use wave::Wave;
use viewer::*;
//use load::test::TestLoader;
use load::vcd::VcdLoader;
use scripts::{
    lua::LuaInterpreter
};
use pipeline::*;

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
    let loader = VcdLoader::new(PathBuf::from(opts.input), opts.clock_period)?;
    let source_adapter = SourceAdapter { };   // FIXME
    let exit_adapter = ExitAdapter {};  // FIXME
    let wave = Wave::load_new(loader, source_adapter, exit_adapter)?;
    let mut state = State::new();
    //let mut interpreter = LuaInterpreter::new(state, wave);
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
            state.resize(stack[0].width - 48, stack[0].height - 2);
            state.data_size(wave.num_signals(), wave.num_cycles());
            
            let table = build_table(&wave, &state);
            f.render_stateful_widget(table, size, state.get_mut_table_state());

            let statusline = build_statusline(&state);
            f.render_widget(statusline, stack[1]);

            let commandline = build_commandline(&state);
            f.render_widget(commandline, stack[2]);
        })?;

        // check events
        if event::poll(Duration::from_millis(200))? {
            let ev = event::read()?;
            //let state = interpreter.state_mut();

            if state.in_command() {
                match ev {
                    Event::Key(KeyEvent { code: KeyCode::Char(c), .. }) => {
                        state.put_command(c);
                    }

                    Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                        state.exec_command(&mut interpreter)?;
                    }

                    Event::Key(KeyEvent { code: KeyCode::Backspace, .. }) => {
                        state.take_command();
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
                        state.move_cursor_down();
                    }

                    // page down
                    Event::Key(KeyEvent { code: KeyCode::Char('J'), .. }) => {
                        state.move_page_down();
                    }

                    // up
                    Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => {
                        state.move_cursor_up();
                    }
                    
                    // page up
                    Event::Key(KeyEvent { code: KeyCode::Char('K'), .. }) => {
                        state.move_page_up();
                    }
                    
                    // left
                    Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => {
                        state.move_cursor_left();
                    }

                    // page left
                    Event::Key(KeyEvent { code: KeyCode::Char('H'), .. }) => {
                        state.move_page_left();
                    }

                    // right
                    Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => {
                        state.move_cursor_right();
                    }

                    // page right
                    Event::Key(KeyEvent { code: KeyCode::Char('L'), .. }) => {
                        state.move_page_right();
                    }

                    // zoom in '+'
                    Event::Key(KeyEvent { code: KeyCode::Char('+'), .. }) => {
                        state.zoom_in();
                    }

                    // zoom out '-'
                    Event::Key(KeyEvent { code: KeyCode::Char('-'), .. }) => {
                        state.zoom_out();
                    }

                    Event::Key(KeyEvent { code: KeyCode::Char(':'), .. }) => {
                        state.start_command();
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
