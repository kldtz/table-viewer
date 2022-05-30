//! Table rendering.
use crate::state::CharCoord;
use crate::state::TableState;
use std::cmp::min;
use termion::style;

pub enum RenderingAction {
    MoveCursor,
    Rerender,
    Command,
    Reset,
    None,
}

/// Rendering interface: receives table state and generates rendering string.
pub trait TableRenderer {
    fn render(&self, ts: &TableState, action: &RenderingAction) -> Option<String> {
        match action {
            RenderingAction::Rerender => Some(self.full_render(ts)),
            RenderingAction::MoveCursor => Some(self.go_to_cur_pos(ts)),
            RenderingAction::Command => Some(self.render_command(ts)),
            RenderingAction::Reset => Some(self.reset_window()),
            _ => None,
        }
    }
    fn window_size(&self) -> CharCoord;
    fn full_render(&self, ts: &TableState) -> String;
    fn go_to_cur_pos(&self, ts: &TableState) -> String;
    fn render_command(&self, ts: &TableState) -> String;
    fn reset_window(&self) -> String;
}

/// A table renderer for TTY terminals.
pub struct TerminalTableRenderer;

impl TerminalTableRenderer {
    fn generate_frame(&self, ts: &TableState) -> String {
        let mut lines: Vec<String> = Vec::with_capacity(ts.rows.len() + 1);
        lines.push(self.format_header(ts, &ts.header));
        let stop = min(ts.offsets.row + ts.terminal_size.y - 1, ts.rows.len());
        lines.extend(
            (ts.rows[ts.offsets.row..stop])
                .iter()
                .map(|row| self.format_row(ts, row)),
        );
        format!("{}", lines.join("\r\n"))
    }

    fn format_header(&self, ts: &TableState, row: &[String]) -> String {
        format!(
            "{}{}{}",
            style::Bold,
            self.format_row(ts, row),
            style::Reset
        )
    }
    fn format_row(&self, ts: &TableState, row: &[String]) -> String {
        let mut cells: Vec<String> = Vec::with_capacity(ts.columns.len() - ts.offsets.col);
        for i in ts.offsets.col..ts.columns.len() {
            let column = &ts.columns[i];
            let value = &row[i];
            if column.index >= ts.terminal_size.x + ts.x_offset() {
                break;
            }
            let last_col_pos = column.index + column.width - ts.x_offset();
            let width = if last_col_pos > ts.terminal_size.x {
                column.width - (last_col_pos - ts.terminal_size.x)
            } else {
                column.width
            };
            cells.push(fixed_width(value, width));
        }
        cells.join("")
    }
}

impl TableRenderer for TerminalTableRenderer {
    fn window_size(&self) -> CharCoord {
        let (x, y) = termion::terminal_size().unwrap();
        CharCoord {
            x: x as usize,
            y: y as usize,
        }
    }

    fn reset_window(&self) -> String {
        format!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1))
    }

    fn full_render(&self, ts: &TableState) -> String {
        format!("{}{}{}", self.reset_window(), self.generate_frame(ts), self.go_to_cur_pos(ts))
    }

    fn go_to_cur_pos(&self, ts: &TableState) -> String {
        format!(
            "{}",
            termion::cursor::Goto(
                (ts.columns[ts.offsets.col + ts.cur_pos.col].index - ts.x_offset() + 1) as u16,
                ts.cur_pos.row as u16 + 1
            )
        )
    }

    fn render_command(&self, ts: &TableState) -> String {
        format!(
            "{}{}{}{}",
            termion::cursor::Goto(1 as u16, ts.terminal_size.y as u16),
            (0..ts.terminal_size.x).map(|_| " ").collect::<String>(),
            termion::cursor::Goto(1 as u16, ts.terminal_size.y as u16),
            ts.command_buffer.iter().collect::<String>(),
        )
    }
}

fn fixed_width(value: &str, col_width: usize) -> String {
    if value.len() > col_width {
        format!("{}…", &value[0..col_width - 1])
    } else {
        format!("{:width$}", value, width = col_width)
    }
}
