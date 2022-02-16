use viow::*;
use clap::Parser;
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::rc::Rc;
use std::io;

#[cfg(not(tarpaulin_include))]
fn main() -> error::Result<()> {
    let opts: Opts = Opts::parse();
    let config = Rc::new(config::Config::load());
    let mut stdout = io::stdout();

    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    match main_loop(stdout, opts, config) {
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
