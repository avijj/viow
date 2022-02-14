use viow::{Opts, setup, config};
use clap::Parser;
use std::rc::Rc;
use std::fs;
use std::env;

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

