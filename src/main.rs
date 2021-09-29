use std::io;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{
    Widget, Block, Borders, Table, TableState, Row, Cell
};
use tui::layout::{
    Layout, Constraint, Direction, Rect
};
use tui::style::{Style, Color, Modifier};
use tui::text::{Spans, Span};
use crossterm::event::{self, Event, KeyEvent, KeyCode};
use std::time::Duration;
use ndarray::prelude::*;

struct State {
    wave_cols: usize,
    wave_rows: usize,
    cur_wave_row: usize,
    cur_wave_col: usize,
    table_state: TableState,
}

impl State {
    fn new() -> Self {
        Self {
            wave_cols: 1,
            wave_rows: 1,
            cur_wave_row: 0,
            cur_wave_col: 0,
            table_state: TableState::default(),
        }
    }

    fn resize(&mut self, wave_width: u16, wave_height: u16) {
        self.wave_cols = wave_width as usize;
        self.wave_rows = wave_height as usize;
    }

    fn get_mut_table_state(&mut self) -> &mut TableState {
        &mut self.table_state
    }

    fn get_cur_wave_col(&self) -> usize {
        self.cur_wave_col
    }

    fn get_cur_wave_row(&self) -> usize {
        self.cur_wave_row
    }

    fn move_cursor_left(&mut self) {
        if self.cur_wave_col > 0 {
            self.cur_wave_col -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cur_wave_col < self.wave_cols - 1 {
            self.cur_wave_col += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            if sel > 0 {
                self.table_state.select(Some(sel-1));
            }
        } else {
            self.table_state.select(Some(self.wave_rows - 1));
        }
    }

    fn move_cursor_down(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            if sel < self.wave_rows - 1 {
                self.table_state.select(Some(sel+1));
            }
        } else {
            self.table_state.select(Some(0));
        }
    }
}


fn format_bit(value: &u8) -> char {
    match value {
        0 => '▁',
        _ => '▇',
    }
}

//fn format_vec(s: String, value: &u8) -> String {
//}

fn build_table<'a>(data : &'a Array2::<u8>, size: &'_ Rect, state: &State) -> Table<'a> {
    let even_style = Style::default()
        .fg(Color::Black)
        .bg(Color::White);
    let odd_style = Style::default()
        .fg(Color::Black)
        .bg(Color::Gray);
    let hi_style = Style::default()
        .bg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let cursor_style = Style::default()
        .bg(Color::Green)
        .add_modifier(Modifier::BOLD);


    let (_num_cycles, num_signals) = data.dim();
    let mut rows = Vec::with_capacity(num_signals);

    let num_signals = std::cmp::min((size.height - 2) as usize, num_signals);

    for row_i in 0..num_signals {
        let cur_cycle = state.get_cur_wave_col();
        let s_pre: String = data.slice(s![..cur_cycle, row_i]).iter()
            .map(format_bit)
            .take(size.width as usize)
            .collect();
        let s_cur: String = data.slice(s![cur_cycle, row_i]).iter()
            .map(format_bit)
            .take(size.width as usize)
            .collect();
        let s_post: String = data.slice(s![cur_cycle+1.., row_i]).iter()
            .map(format_bit)
            .take(size.width as usize)
            .collect();

        let ref cur_style = if row_i % 2 == 0 { even_style } else { odd_style };

        let name_cell = Cell::from(format!("row_{}", row_i))
            .style(*cur_style);
        let value_cell = Cell::from(format!("{}", data[[cur_cycle, row_i]]))
            .style(*cur_style);
        let wave_cell = Cell::from(Spans::from(vec![
                Span::raw(s_pre),
                Span::styled(s_cur, cursor_style),
                Span::raw(s_post)
            ]))
            .style(*cur_style);

        rows.push(Row::new(vec![name_cell, value_cell, wave_cell]));
    }

    Table::new(rows)
        .header(Row::new(vec!["Name", "Value", "Waveform"])
            .style(Style::default().fg(Color::Yellow))
            .bottom_margin(0))
        .widths(&[Constraint::Min(40), Constraint::Length(8), Constraint::Ratio(1, 1)])
        .column_spacing(0)
        .highlight_style(hi_style)
}

fn main() -> Result<(),io::Error> {
    crossterm::terminal::enable_raw_mode()
        .expect("Can't run in raw mode");

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let data = {
        let mut data = Array2::<u8>::zeros((1000, 200));
        data.slice_mut(s![..,1]).fill(1);
        data.slice_mut(s![..;2,2]).fill(1);
        let counter: Vec<u8> = (0..data.dim().0).into_iter()
            .map(|x: usize| ((x >> 2) % 16) as u8)
            .collect();
        data.slice_mut(s![..,4]).assign(&Array1::from_vec(counter));

        data
    };

    let mut state = State::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            state.resize(size.width - 48, size.height - 2);
            let table = build_table(&data, &size, &state);

            f.render_stateful_widget(table, size, state.get_mut_table_state());

        })?;

        // check events
        if event::poll(Duration::from_millis(200))? {
            let ev = event::read()?;
            match ev {
                // quit
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                    break
                }

                // down
                Event::Key(KeyEvent { code: KeyCode::Char('j'), .. }) => {
                    state.move_cursor_down();
                }

                // up
                Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => {
                    state.move_cursor_up();
                }
                
                // left
                Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => {
                    state.move_cursor_left();
                }

                // right
                Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => {
                    state.move_cursor_right();
                }
                _ => {}
            }
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap();

    Ok(())
}
