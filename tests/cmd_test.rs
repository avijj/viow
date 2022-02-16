use viow::{Opts, setup, render_loop, config};
use clap::Parser;
use std::rc::Rc;
use std::fs;
use std::env;
use std::thread;
use std::time;
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

/// Check if we can open all files from examples directory.
#[test]
fn open_examples() {
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
}


/*#[test]
fn interactive() {
    const EXAMPLE_FILE: &'static str = "filter_test.lua";

    env::set_current_dir("examples/").unwrap();

    let mut stdout = std::io::stdout();
    let args = [
        String::from("viow"),
        String::from("--cycle-step"), String::from("100"),
        String::from("--timeunits"), String::from("ps"),
        EXAMPLE_FILE.to_string()
    ];
    let opts = Opts::parse_from(&args);
    let config = Rc::new(config::Config::test_config());

    thread::spawn(move || {
        let mut stdin = std::io::stdin();
        thread::sleep(time::Duration::from_millis(500));

    });

    stdout.execute(EnterAlternateScreen).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();

    render_loop(stdout, opts, config).unwrap();

    crossterm::terminal::disable_raw_mode().unwrap();
    std::io::stdout().execute(LeaveAlternateScreen).unwrap();
}*/

