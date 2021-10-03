mod wave;
mod formatting;
mod load;

use wave::Wave;
use formatting::build_waveform;
use load::test::TestLoader;

use std::io;
use tui::Terminal;
use tui::backend::CrosstermBackend;
use tui::widgets::{
    Table, TableState, Row, Cell, Paragraph
};
use tui::layout::{
    Layout, Constraint, Direction 
};
use tui::style::{Style, Color, Modifier};
use tui::text::{Spans, Span};
use crossterm::{
    ExecutableCommand,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyEvent, KeyCode}
};
use std::time::Duration;


struct State {
    /// Visible cols in waveform view
    wave_cols: usize,

    /// Visible rows in waveform view
    wave_rows: usize,

    /// Total number of columns in data
    data_cols: usize,

    /// Total number of rows in data
    data_rows: usize,

    /// Top row in data matching row 0 in waveform view
    top_wave_row: usize,

    /// Leftmost column in data matching col 0 in waveform view
    left_wave_col: usize,

    /// Row where cursor is
    cur_wave_row: usize,

    /// Column where cursor is
    cur_wave_col: usize,

    /// State of the Table widget
    table_state: TableState,

    /// Number of columns in view for a data column
    zoom: usize,
}

impl State {
    fn new() -> Self {
        Self {
            wave_cols: 1,
            wave_rows: 1,
            data_cols: 0,
            data_rows: 0,
            top_wave_row: 0,
            left_wave_col: 0,
            cur_wave_row: 0,
            cur_wave_col: 0,
            table_state: TableState::default(),
            zoom: 1,
        }
    }

    fn resize(&mut self, wave_width: u16, wave_height: u16) {
        self.wave_cols = wave_width as usize / self.zoom;
        self.wave_rows = wave_height as usize;
    }

    fn data_size(&mut self, rows: usize, cols: usize) {
        self.data_rows = rows;
        self.data_cols = cols;
    }

    fn get_mut_table_state(&mut self) -> &mut TableState {
        &mut self.table_state
    }

    //fn get_cur_wave_col(&self) -> usize {
        //self.cur_wave_col
    //}

    //fn get_cur_wave_row(&self) -> usize {
        //self.cur_wave_row
    //}

    fn move_cursor_left(&mut self) {
        if self.cur_wave_col > 0 {
            if self.cur_wave_col == self.left_wave_col {
                self.left_wave_col -= 1;
            }
            self.cur_wave_col -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cur_wave_col < self.data_cols - 1 {
            if self.cur_wave_col == self.left_wave_col + self.wave_cols - 1 {
                self.left_wave_col += 1;
            }
            self.cur_wave_col += 1;
        }
    }

    fn move_cursor_up(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            if sel > 0 {
                self.table_state.select(Some(sel-1));
                self.cur_wave_row -= 1;
            } else if self.cur_wave_row > 0 {
                self.top_wave_row -= 1;
                self.cur_wave_row -= 1;
            }
        } else {
            self.table_state.select(Some(self.wave_rows - 1));
            self.cur_wave_row = self.data_rows - 1;
            self.top_wave_row = self.data_rows - self.wave_rows;
        }
    }

    fn move_cursor_down(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            if sel < self.wave_rows - 1 {
                self.table_state.select(Some(sel+1));
                self.cur_wave_row += 1;
            } else if self.cur_wave_row < self.data_rows - 1 {
                self.top_wave_row += 1;
                self.cur_wave_row += 1;
            }
        } else {
            self.table_state.select(Some(0));
            self.top_wave_row = 0;
            self.cur_wave_row = 0;
        }
    }

    fn move_page_down(&mut self) {
        if self.top_wave_row < self.data_rows - 1 {
            let inc = std::cmp::min(self.data_rows - self.top_wave_row - self.wave_rows, self.wave_rows);
            self.top_wave_row += inc;
            self.cur_wave_row = self.top_wave_row;
            self.table_state.select(Some(0));
        }
    }

    fn move_page_up(&mut self) {
        if self.top_wave_row > 0 {
            let dec = std::cmp::min(self.top_wave_row, self.wave_rows);
            self.top_wave_row -= dec;
            self.cur_wave_row = self.top_wave_row + self.wave_rows - 1;
            self.table_state.select(Some(self.wave_rows - 1));
        }
    }

    fn move_page_right(&mut self) {
        if self.left_wave_col < self.data_cols - 1 {
            let inc = std::cmp::min(self.data_cols - self.left_wave_col - self.wave_cols, self.wave_cols);
            self.left_wave_col += inc;
            self.cur_wave_col = self.left_wave_col;
        }
    }

    fn move_page_left(&mut self) {
        if self.left_wave_col > 0 {
            let dec = std::cmp::min(self.left_wave_col, self.wave_cols);
            self.left_wave_col -= dec;
            self.cur_wave_col = self.left_wave_col + self.wave_cols - 1;
        }
    }

    fn zoom_in(&mut self) {
        self.zoom += 1;
    }

    fn zoom_out(&mut self) {
        if self.zoom > 1 {
            self.zoom -= 1;
        }
    }
}



fn build_table<'a>(wave: &'a Wave, state: &State) -> Table<'a> {
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


    let mut rows = Vec::with_capacity(state.wave_rows);

    let top = state.top_wave_row;
    let bot = state.top_wave_row + state.wave_rows;
    let left = state.left_wave_col;
    let right = state.left_wave_col + state.wave_cols;

    for row_i in top..bot {
        let fmt = build_waveform(wave.slice_of_signal(row_i, left, right), wave.formatter(row_i), state.zoom);
        let cur_cycle = (state.cur_wave_col - state.left_wave_col) * state.zoom;
        let s_pre: String = fmt.chars().take(cur_cycle).collect();
        let s_cur: String = fmt.chars().skip(cur_cycle).take(state.zoom).collect();
        let s_post: String = fmt.chars().skip(cur_cycle+state.zoom).collect();

        let ref cur_style = if row_i % 2 == 0 { even_style } else { odd_style };

        let name_cell = Cell::from(wave.name(row_i).unwrap())
            .style(*cur_style);
        //let name_cell = Cell::from(format!("row_{}", row_i))
            //.style(*cur_style);
        let value_cell = Cell::from(format!("0x{:>8x}", wave.value(row_i, cur_cycle).unwrap()))
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
        .widths(&[Constraint::Min(37), Constraint::Length(11), Constraint::Ratio(1, 1)])
        .column_spacing(0)
        .highlight_style(hi_style)
}

fn build_statusline(state: &State) -> Paragraph {
    let line_txt = vec![
        Spans::from(vec![
            Span::raw(format!("Cursor: {},{}", state.cur_wave_row, state.cur_wave_col))
        ])
    ];

    Paragraph::new(line_txt)
}

fn main() -> Result<(),io::Error> {
    let mut stdout = io::stdout();
    
    stdout.execute(EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    /*let (data, formatters) = {
        //let mut data = vec![vec![Integer::from(0); 200]];
        //let mut data = Array2::<Integer>::zeros((1000, 200));
        let mut data = Array2::<Integer>::from_elem((1000, 200), Integer::from(0));
        let mut formatters = vec![WaveFormat::Bit; 200];
        data.slice_mut(s![..,1]).fill(Integer::from(1));
        data.slice_mut(s![..;2,2]).fill(Integer::from(1));

        let counter: Vec<Integer> = (0..data.dim().0).into_iter()
            .map(|x: usize| Integer::from((x >> 2) % 16))
            .collect();
        data.slice_mut(s![..,4]).assign(&Array1::from_vec(counter));
        formatters[4] = WaveFormat::Vector;

        let counter: Vec<Integer> = (0x4000..data.dim().0 + 0x4000).into_iter()
            .map(|x: usize| Integer::from((x >> 2) % 0x10000))
            .collect();
        data.slice_mut(s![..,5]).assign(&Array1::from_vec(counter));
        formatters[5] = WaveFormat::Vector;

        (data, formatters)
    };*/

    //let wave = Wave::new();
    let loader = TestLoader::new(200, 2000);
    let wave = Wave::load(&loader);
    let mut state = State::new();

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let stack = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1)
                ])
                .split(size);

            state.resize(stack[0].width - 48, stack[0].height - 2);
            state.data_size(wave.num_cycles(), wave.num_signals());
            let table = build_table(&wave, &state);

            f.render_stateful_widget(table, size, state.get_mut_table_state());

            let statusline = build_statusline(&state);
            f.render_widget(statusline, stack[1]);
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

                _ => {}
            }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(LeaveAlternateScreen)?;

    Ok(())
}
