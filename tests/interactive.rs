use crossterm::event::KeyModifiers;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use viow::{Opts, setup, render_step, event_step, config};
use clap::Parser;
use std::rc::Rc;
use std::env;
use crossterm::{
    event::Event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

fn char_key(c: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(c),
        modifiers: KeyModifiers::empty()
    })
}

/// Test a single render step.
#[test]
fn interactive() {
    const EXAMPLE_FILES: [&'static str; 2] = [
        "filter_test.lua",
        "no_filter.lua"
    ];

    let stimulus = [
        // filter_test.lua
        [
            vec![char_key('j'); 20],
            vec![char_key('+'), char_key('-')],
            vec![char_key('l'); 200],
            vec![char_key('h'); 200],
            vec![char_key('k'); 20],
            vec![char_key('q')]
        ].concat(),

        // no_filter.lua
        [
            vec![char_key('J'); 20],
            vec![char_key('L'); 200],
            vec![char_key('H'); 200],
            vec![char_key('K'); 20],
            vec![char_key('q')]
        ].concat(),
    ];

    let save_cwd = env::current_dir().unwrap();
    env::set_current_dir("examples/").unwrap();

    for (file,stim) in EXAMPLE_FILES.iter().zip(stimulus) {
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

        let mut step = setup(opts, config).unwrap();

        for event in stim {
            step = render_step(&mut terminal, step).unwrap();
            step = event_step(step, event).unwrap();
        }

        crossterm::terminal::disable_raw_mode().unwrap();
        std::io::stdout().execute(LeaveAlternateScreen).unwrap();
    }

    env::set_current_dir(save_cwd).unwrap();
}


