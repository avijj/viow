use tui::Terminal;
use tui::backend::CrosstermBackend;
use viow::{Opts, setup, render_step, config};
use clap::Parser;
use std::rc::Rc;
use std::fs;
use std::env;
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

/// Check if we can open all files from examples directory.
fn open_examples() {
    let save_cwd = env::current_dir().unwrap();
    env::set_current_dir("examples/").unwrap();

    for entry in fs::read_dir("./").unwrap() {
        let entry = entry.unwrap();
        
        if entry.file_type().unwrap().is_file() {
            let path = entry.path().to_str().unwrap().to_string();
            let args = [
                String::from("viow"),
                String::from("--cycle-step"), String::from("100"),
                String::from("--timeunits"), String::from("ps"),
                path
            ];

            let opts = Opts::parse_from(&args);
            let config = Rc::new(config::Config::test_config());

            let _ = setup(opts, config)
                .expect("Failed setup()");

        }
    }

    env::set_current_dir(save_cwd).unwrap();
}


/// Test a single render step.
fn render() {
    const EXAMPLE_FILES: [&'static str; 2] = [
        "filter_test.lua",
        "no_filter.lua"
    ];

    let save_cwd = env::current_dir().unwrap();
    env::set_current_dir("examples/").unwrap();

    for file in EXAMPLE_FILES {
        let mut stdout = std::io::stdout();
        let args = [
            String::from("viow"),
            String::from("--cycle-step"), String::from("100"),
            String::from("--timeunits"), String::from("ps"),
            file.to_string()
        ];
        let opts = Opts::parse_from(&args);
        let config = Rc::new(config::Config::test_config());

        stdout.execute(EnterAlternateScreen).unwrap();
        crossterm::terminal::enable_raw_mode().unwrap();

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        let step = setup(opts, config).unwrap();
        render_step(&mut terminal, step).unwrap();

        crossterm::terminal::disable_raw_mode().unwrap();
        std::io::stdout().execute(LeaveAlternateScreen).unwrap();
    }

    env::set_current_dir(save_cwd).unwrap();
}


/// Combined test
///
/// Tests use environment variables and current working directory and so have to run sequentially.
#[test]
fn cmd_test() {
    open_examples();
    render();
}
