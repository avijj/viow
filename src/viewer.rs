use crate::error::*;
use crate::formatting::build_waveform;
use crate::wave::Wave;
use crate::config::Config;

use tui::widgets::*;
use tui::terminal::Frame;
use tui::backend::Backend;
use tui::layout::{Direction, Layout, Rect, Constraint};
use tui::style::{Style, Color, Modifier};
use tui::text::{Text, Spans, Span};
use rustyline;

use std::rc::Rc;

type ReadlineEditor = rustyline::Editor<()>;

const MAX_NAME_COL_WIDTH: u16 = 100;
const MAX_VALUE_COL_WIDTH: u16 = 40;

#[derive(Debug)]
pub struct InsertState {
    prompt: String,
    list: Vec<String>,
    at: usize,
    list_state: ListState,
    signals: Vec<String>,
    suggested: Vec<String>,
    suggestion_state: ListState,
}

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert(InsertState),
}

#[derive(Debug)]
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

    /// Readline editor
    line_editor: ReadlineEditor,
}

impl State {
    pub fn new(config: &Rc<Config>) -> Result<Self> {
        let mut line_editor = ReadlineEditor::with_config(config.readline_config().clone());
        if let Some(history) = config.readline_history() {
            line_editor.load_history(history)?;
        }

        Ok(Self {
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
            line_editor,
        })
    }

    pub fn save(&mut self, config: &Rc<Config>) -> Result<()> {
        if let Some(history) = config.readline_history() {
            self.line_editor.append_history(history)?;
        }

        Ok(())
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
        if x >= self.data_cols {
            self.cur_wave_col = self.data_cols.saturating_sub(1);
            self.left_wave_col = self.data_cols.saturating_sub(self.wave_cols - 1);
        } else {
            self.cur_wave_col = x;
            self.left_wave_col = x.saturating_sub(self.wave_cols/2);
        }
    }

    pub fn get_cursor_row(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub fn get_cur_wave_row(&self) -> usize {
        self.cur_wave_row
    }

    pub fn set_cur_wave_row(&mut self, x: Option<usize>) {
        if let Some(x) = x {
            if x < self.data_rows {
                self.cur_wave_row = x;
                self.top_wave_row = x.saturating_sub(self.wave_rows/2);
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
        } else {
            self.cur_wave_row = 0;
            self.top_wave_row = 0;
            self.table_state.select(None);
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
        self.set_cur_wave_row(Some(self.cur_wave_row.saturating_add(self.wave_rows)));
    }

    pub fn move_page_up(&mut self) {
        self.set_cur_wave_row(Some(self.cur_wave_row.saturating_sub(self.wave_rows)));
    }

    pub fn move_page_right(&mut self) {
        self.set_cur_wave_col(self.cur_wave_col.saturating_add(self.wave_cols));
    }

    pub fn move_page_left(&mut self) {
        self.set_cur_wave_col(self.cur_wave_col.saturating_sub(self.wave_cols));
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

    pub fn line_editor_mut(&mut self) -> &mut ReadlineEditor {
        &mut self.line_editor
    }
    
    pub fn start_insert_mode(&mut self, signals: Vec<String>, mut list: Vec<String>, insert_at: usize) {
        list.insert(insert_at, "#".into());
        let mut list_state = ListState::default();
        list_state.select(Some(insert_at));

        let m = InsertState {
            prompt: "".into(),
            list,
            list_state,
            signals,
            at: insert_at,
            suggested: vec![],
            suggestion_state: ListState::default(),
        };

        self.mode = Mode::Insert(m);
    }

    pub fn in_insert_mode(&self) -> bool {
        match self.mode {
            Mode::Insert(_) => true,
            _ => false,
        }
    }

    pub fn exit_insert_mode(&mut self) -> Result<String> {
        let rv;

        match self.mode {
            Mode::Insert(ref istate) => {
                rv = istate.prompt.clone();
            }

            _ => {
                return Err(Error::WrongMode("Expect to be in insert mode, when exiting insert mode".into()));
            }
        }
        
        self.mode = Mode::Normal;
        Ok(rv)
    }

    pub fn put_key(&mut self, c: char) {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                istate.prompt.push(c);
                Self::update_suggestions(istate);
            }

            _ => { }
        }
    }

    pub fn take_key(&mut self) -> Option<char> {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                let rv = istate.prompt.pop();
                Self::update_suggestions(istate);
                rv
            }

            _ => {
                None
            }
        }
    }

    pub fn enter_key(&mut self) {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                if istate.at + 1 >= istate.list.len() {
                    istate.list.push("#".into());
                } else {
                    istate.list.insert(istate.at + 1, "#".into());
                }
                istate.at += 1;
                istate.list_state.select(Some(istate.at));
                istate.suggestion_state.select(None);
            }

            _ => {}
        }
    }

    fn update_suggestions(istate: &mut InsertState) -> usize {
        let suggested = istate.signals.iter() 
            .filter_map(|x| {
                if x.contains(&istate.prompt) {
                    Some(x.clone())
                } else {
                    None
                }
            })
            .collect();

        if let Some(selected) = istate.suggestion_state.selected() {
            if selected >= istate.suggested.len() {
                istate.suggestion_state.select(None);
            }
        }

        istate.suggested = suggested;
        istate.suggested.len()
    }

    pub fn next_suggestion(&mut self) {
        match self.mode {
            Mode::Insert(ref mut istate) => {
                if let Some(selected) = istate.suggestion_state.selected() {
                    if selected + 1 < istate.suggested.len() {
                        istate.suggestion_state.select(Some(selected + 1));
                        istate.list[istate.at] = istate.suggested[selected + 1].clone();
                    } else {
                        istate.suggestion_state.select(None);
                        istate.list[istate.at] = "#".into();
                    }
                } else {
                    if !istate.suggested.is_empty() {
                        istate.suggestion_state.select(Some(0));
                        istate.list[istate.at] = istate.suggested[0].clone();
                    } else {
                        istate.list[istate.at] = "#".into();
                    }
                }
            }

            _ => {}
        }
    }

    pub fn get_insert_list(&self) -> Option<&Vec<String>> {
        match self.mode {
            Mode::Insert(ref istate) => {
                Some(&istate.list)
            }

            _ => {
                None
            }
        }
    }
}


pub fn build_table<'a>(wave: &'a mut Wave, state: &State) -> (u16, u16, Table<'a>) {
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

    let mut max_name_width = 0u16;
    let mut max_value_width = 0u16;

    let wave_slice = wave.cached_slice(top..bot, left..right)
        .unwrap();   // Can't report error, because called from tui drawing closure.

    for row_i in top..bot {
        let signal_slice = wave_slice.signal_iter(row_i)
            .unwrap();  // should not happen, due to for loop limits
        let fmt = build_waveform(signal_slice, wave_slice.formatter(row_i), state.zoom);
        let cur_cycle = (state.cur_wave_col - state.left_wave_col) * state.zoom;
        let s_pre: String = fmt.chars().take(cur_cycle).collect();
        let s_cur: String = fmt.chars().skip(cur_cycle).take(state.zoom).collect();
        let s_post: String = fmt.chars().skip(cur_cycle+state.zoom).collect();

        let ref cur_style = if row_i % 2 == 0 { even_style } else { odd_style };

        let name = wave_slice.name(row_i).unwrap_or("⁇⁇⁇");
        if name.len() as u16 > max_name_width {
            max_name_width = name.len() as u16;
        }
        let name_cell = Cell::from(name)
            .style(*cur_style);

        let value_txt = wave_slice.formatted_value(row_i, state.cur_wave_col)
            .unwrap_or("⁇".to_string());
        if value_txt.len() as u16 > max_value_width {
            max_value_width = value_txt.len() as u16;
        }
        let value_cell = Cell::from(value_txt)
            .style(*cur_style);

        let wave_cell = Cell::from(Spans::from(vec![
                Span::raw(s_pre),
                Span::styled(s_cur, cursor_style),
                Span::raw(s_post)
            ]))
            .style(*cur_style);

        rows.push(Row::new(vec![name_cell, value_cell, wave_cell]));
    }

    // upper bound for column width
    max_name_width = std::cmp::min(max_name_width, MAX_NAME_COL_WIDTH);
    max_value_width = std::cmp::min(max_value_width, MAX_VALUE_COL_WIDTH);

    // This is necessary due to tui's API. We need to pass constraint as reference. We can't pass a
    // reference to function owned variable.
    (
        max_name_width,
        max_value_width,
        Table::new(rows)
            .header(Row::new(vec!["Name", "Value", "Waveform"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(0))
            .column_spacing(1)
            .highlight_style(hi_style)
    )
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

pub fn render_insert<T: Backend>(frame: &mut Frame<T>, rect: &Rect, state: &mut State) {
    if let Mode::Insert(ref mut insert_state) = state.mode {
        //
        // Construct widgets
        //

        //let name_list = &wave.get_config().name_list;
        let name_list = &insert_state.list;
        let items: Vec<_> = name_list.iter()
            .map(|name| ListItem::new(name.as_ref()))
            .collect();

        let prompt_line = Paragraph::new(Text::raw(&insert_state.prompt));
        let suggestion_items: Vec<_> = insert_state.suggested.iter()
            .map(|name| ListItem::new(name.clone()))
            .collect();

        let suggestion_list = List::new(suggestion_items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol(">>");

        let list = List::new(items)
            .highlight_symbol(">");

        //
        // Drawing
        //

        let vsplit = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([
                Constraint::Ratio(1,2),
                Constraint::Ratio(1,2)
            ])
            .split(*rect);
        let right = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(vsplit[1]);

        frame.render_stateful_widget(list, vsplit[0], &mut insert_state.list_state);
        frame.render_widget(prompt_line, right[0]);
        frame.render_stateful_widget(suggestion_list, right[1], &mut insert_state.suggestion_state);
    }
}
