use crate::formatting::build_waveform;
use crate::wave::Wave;

use tui::widgets::TableState;
use tui::widgets::{ List, ListItem, Table, Row, Cell, Paragraph };
use tui::layout::Constraint;
use tui::style::{Style, Color, Modifier};
use tui::text::{Spans, Span};

pub struct InsertState {
    prompt: String,
}

pub enum Mode {
    Normal,
    Insert(InsertState),
}

pub struct State {
    /// Interaction mode currently active
    mode: Mode,

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

    /// State of the command buffer
    command: Option<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            mode: Mode::Normal,
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
            command: None,
        }
    }

    pub fn resize(&mut self, wave_width: u16, wave_height: u16) {
        self.wave_cols = wave_width as usize / self.zoom;
        self.wave_rows = wave_height as usize;
    }

    pub fn data_size(&mut self, rows: usize, cols: usize) {
        self.data_rows = rows;
        self.data_cols = cols;
    }

    pub fn get_mut_table_state(&mut self) -> &mut TableState {
        &mut self.table_state
    }

    pub fn get_cur_wave_col(&self) -> usize {
        self.cur_wave_col
    }

    pub fn set_cur_wave_col(&mut self, x: usize) {
        if x < self.left_wave_col {
            self.left_wave_col = x;
        }

        if x >= self.data_cols {
            self.cur_wave_col = self.data_cols.saturating_sub(1);
            self.left_wave_col = self.data_cols.saturating_sub(1 + self.wave_cols);
        } else {
            self.cur_wave_col = x;
            self.left_wave_col = x.saturating_sub(self.wave_cols-1);
        }
    }

    pub fn get_cur_wave_row(&self) -> usize {
        self.cur_wave_row
    }

    pub fn set_cur_wave_row(&mut self, x: usize) {
        if x < self.data_rows {
            self.cur_wave_row = x;
            self.top_wave_row = x.saturating_sub(self.wave_rows);
            self.table_state.select(Some(self.cur_wave_row - self.top_wave_row));
        } else {
            let num_rows = std::cmp::min(self.wave_rows, self.data_rows);

            if num_rows > 0 {
                self.table_state.select(Some(num_rows - 1));
            } else {
                self.table_state.select(None);
            }
            self.cur_wave_row = self.data_rows.saturating_sub(1);
            self.top_wave_row = self.data_rows.saturating_sub(self.wave_rows);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cur_wave_col > 0 {
            if self.cur_wave_col == self.left_wave_col {
                self.left_wave_col -= 1;
            }
            self.cur_wave_col -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cur_wave_col + 1 < self.data_cols {
            if self.cur_wave_col == self.left_wave_col + self.wave_cols - 1 {
                self.left_wave_col += 1;
            }
            self.cur_wave_col += 1;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            if sel > 0 {
                self.table_state.select(Some(sel-1));
                self.cur_wave_row -= 1;
            } else if self.cur_wave_row > 0 {
                self.top_wave_row -= 1;
                self.cur_wave_row -= 1;
            }
        } else {
            let last_row = std::cmp::min(self.wave_rows, self.data_rows) - 1;

            self.table_state.select(Some(last_row));
            self.cur_wave_row = self.data_rows - 1;
            self.top_wave_row = self.data_rows.saturating_sub(self.wave_rows);
        }
    }

    pub fn move_cursor_down(&mut self) {
        if let Some(sel) = self.table_state.selected() {
            let num_rows = std::cmp::min(self.wave_rows, self.data_rows);

            if sel + 1 < num_rows {
                self.table_state.select(Some(sel+1));
                self.cur_wave_row += 1;
            } else if self.cur_wave_row + 1 < self.data_rows {
                self.top_wave_row += 1;
                self.cur_wave_row += 1;
            }
        } else {
            self.table_state.select(Some(0));
            self.top_wave_row = 0;
            self.cur_wave_row = 0;
        }
    }

    pub fn move_page_down(&mut self) {
        if self.top_wave_row < self.data_rows - 1 {
            let inc = std::cmp::min(self.data_rows - self.top_wave_row - self.wave_rows, self.wave_rows);
            self.top_wave_row += inc;
            self.cur_wave_row = self.top_wave_row;
            self.table_state.select(Some(0));
        }
    }

    pub fn move_page_up(&mut self) {
        if self.top_wave_row > 0 {
            let dec = std::cmp::min(self.top_wave_row, self.wave_rows);
            self.top_wave_row -= dec;
            self.cur_wave_row = self.top_wave_row + self.wave_rows - 1;
            self.table_state.select(Some(self.wave_rows - 1));
        }
    }

    pub fn move_page_right(&mut self) {
        if self.left_wave_col + self.wave_cols < self.data_cols - 1 {
            let inc = self.data_cols - self.left_wave_col - self.wave_cols;
            self.left_wave_col += inc;
            self.cur_wave_col = self.left_wave_col;
        }
    }

    pub fn move_page_left(&mut self) {
        if self.left_wave_col > 0 {
            let dec = std::cmp::min(self.left_wave_col, self.wave_cols);
            self.left_wave_col -= dec;
            self.cur_wave_col = self.left_wave_col + self.wave_cols - 1;
        }
    }

    pub fn zoom_in(&mut self) {
        let left_to_cur = (self.cur_wave_col - self.left_wave_col) * self.zoom;
        //self.zoom += 1;
        self.zoom *= 2;
        self.left_wave_col = std::cmp::min(
            self.cur_wave_col - (left_to_cur / self.zoom),
            if self.data_cols >= self.wave_cols { self.data_cols - self.wave_cols } else { 0 }
        );
    }

    pub fn zoom_out(&mut self) {
        if self.zoom > 1 {
            let left_to_cur = (self.cur_wave_col - self.left_wave_col) * self.zoom;
            //self.zoom -= 1;
            self.zoom /= 2;
            self.left_wave_col = std::cmp::min(
                if self.cur_wave_col > (left_to_cur / self.zoom) {
                    self.cur_wave_col - (left_to_cur / self.zoom)
                } else { 
                    0
                },

                if self.data_cols >= self.wave_cols {
                    self.data_cols - self.wave_cols
                } else { 
                    0
                }
            );
        }
    }

    pub fn start_command(&mut self) {
        self.command = Some(String::new());
    }

    pub fn in_command(&self) -> bool {
        self.command.is_some()
    }

    pub fn put_command(&mut self, c: char) {
        if let Some(ref mut txt) = self.command {
            txt.push(c);
        }
    }

    pub fn take_command(&mut self) -> Option<char> {
        if let Some(ref mut txt) = self.command {
            txt.pop()
        } else {
            None
        }
    }

    pub fn pop_command(&mut self) -> Option<String> {
        self.command.take()
    }

    //pub fn exec_command<I: RunCommand>(&mut self, interpreter: &mut I) -> Result<()> {
        //let cmd = self.command.take()
            //.ok_or(Error::NoCommand)?;
        //interpreter.run_command(self, cmd)?;
        //Ok(())
    //}
    
    pub fn start_insert_mode(&mut self) {
        let m = InsertState {
            prompt: "".into(),
        };

        self.mode = Mode::Insert(m);
    }

    pub fn in_insert_mode(&self) -> bool {
        match self.mode {
            Mode::Insert(_) => true,
            _ => false,
        }
    }

    pub fn exit_insert_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn put_key(&mut self, c: char) {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                istate.prompt.push(c);
            }

            _ => { }
        }
    }

    pub fn take_key(&mut self) -> Option<char> {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                istate.prompt.pop()
            }

            _ => {
                None
            }
        }
    }
}


pub fn build_table<'a>(wave: &'a Wave, state: &State) -> Table<'a> {
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
    let bot = std::cmp::min(state.top_wave_row + state.wave_rows, wave.num_signals());
    let left = state.left_wave_col;
    let right = std::cmp::min(state.left_wave_col + state.wave_cols, wave.num_cycles());

    for row_i in top..bot {
        let fmt = build_waveform(wave.slice_of_signal(row_i, left, right), wave.formatter(row_i), state.zoom);
        let cur_cycle = (state.cur_wave_col - state.left_wave_col) * state.zoom;
        let s_pre: String = fmt.chars().take(cur_cycle).collect();
        let s_cur: String = fmt.chars().skip(cur_cycle).take(state.zoom).collect();
        let s_post: String = fmt.chars().skip(cur_cycle+state.zoom).collect();

        let ref cur_style = if row_i % 2 == 0 { even_style } else { odd_style };

        let name_cell = Cell::from(wave.name(row_i).unwrap_or("⁇⁇⁇"))
            .style(*cur_style);
        //let name_cell = Cell::from(format!("row_{}", row_i))
            //.style(*cur_style);
        let value_cell = wave.value(row_i, state.cur_wave_col)
            .map(|val| Cell::from(format!("0x{:>8x}", val)))
            .unwrap_or(Cell::from("⁇"))
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

pub fn build_statusline(state: &State) -> Paragraph {
    let mode_txt = match state.mode {
        Mode::Normal => "  NORMAL",
        Mode::Insert(_) => "  INSERT",
    };

    let line_txt = vec![
        Spans::from(vec![
            Span::raw(format!("Cursor: {},{}", state.cur_wave_row, state.cur_wave_col)),
            Span::raw(mode_txt),
        ])
    ];

    Paragraph::new(line_txt)
}

pub fn build_commandline(state: &State) -> Paragraph {
    let line_txt = vec![
        Spans::from(vec![
            if let Some(ref txt) = state.command {
                Span::raw(format!(":{}", txt))
            } else {
                Span::raw("")
            }
        ])
    ];

    Paragraph::new(line_txt)
}

pub fn build_insert<'a>(wave: &'a Wave, state: &'a State) -> (List<'a>, List<'a>) {
    let name_list = &wave.get_config().name_list;
    let mut items: Vec<_> = name_list.iter()
        .map(|name| ListItem::new(name.as_ref()))
        .collect();

    if let Mode::Insert(ref insert_state) = state.mode {
        let prompt = &insert_state.prompt;
        items.push(ListItem::new(prompt.as_str()));
    }

    (List::new(items), List::new(vec![]))
}
