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

fn format_bit(value: &u8) -> char {
    match value {
        0 => '▁',
        _ => '▇',
    }
}

//fn format_vec(s: String, value: &u8) -> String {
//}

fn build_table<'a>(data : &'a Array2::<u8>, size: &'_ Rect, cur_cycle: usize) -> Table<'a> {
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

    let mut table_state = TableState::default();
    let mut cursor_cycle = 0;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            //let chunks = Layout::default()
                //.direction(Direction::Horizontal)
                //.margin(1)
                //.constraints(
                    //[
                        //Constraint::Percentage(20),
                        //Constraint::Percentage(80)
                    //].as_ref()
                //)
                //.split(f.size());

            //let block = Block::default()
                //.title("Block")
                //.borders(Borders::ALL);
            //f.render_widget(block, chunks[0]);

            //let block2 = Block::default()
                //.title("Block 2")
                //.borders(Borders::ALL);
            //f.render_widget(block2, chunks[1]);

            //let table = Table::new(vec![
                    //Row::new(vec!["clk", "0", "▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇▁▇"]),
                    //Row::new(vec!["foo", "1", "▁▁▁▁▁▁▁▁▁▁▇▇▇▇▇▇▇▇▇▇▇▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁"]),
                    //Row::new(vec!["bar", "1", "000╳001╳010╳011╳100╳101╳110╳111╳000╳001╳00"]),
                //])
                //.header(Row::new(vec!["Name", "Value", "Waveform"])
                    //.style(Style::default().fg(Color::Yellow))
                    //.bottom_margin(1))
                //.widths(&[Constraint::Percentage(20), Constraint::Percentage(10), Constraint::Percentage(70)]);
            let table = build_table(&data, &size, cursor_cycle);

            f.render_stateful_widget(table, size, &mut table_state);

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
                    if let Some(sel) = table_state.selected() {
                        if sel < data.dim().1 -1 {
                            table_state.select(Some(sel+1));
                        }
                    } else {
                        table_state.select(Some(0));
                    }
                }

                // up
                Event::Key(KeyEvent { code: KeyCode::Char('k'), .. }) => {
                    if let Some(sel) = table_state.selected() {
                        if sel > 0 {
                            table_state.select(Some(sel-1));
                        }
                    } else {
                        table_state.select(Some(data.dim().1-1));
                    }
                }
                
                // left
                Event::Key(KeyEvent { code: KeyCode::Char('h'), .. }) => {
                    if cursor_cycle > 0 {
                        cursor_cycle -= 1;
                    }
                }

                // right
                Event::Key(KeyEvent { code: KeyCode::Char('l'), .. }) => {
                    if cursor_cycle < data.dim().0 {
                        cursor_cycle += 1;
                    }
                }
                _ => {}
            }
        }
    }

    crossterm::terminal::disable_raw_mode().unwrap();

    Ok(())
}
