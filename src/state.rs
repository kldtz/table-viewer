//! Table state without external side-effects.
use crate::renderer::RenderingAction;
use core::cmp::Ordering;
use std::cmp::min;
use std::iter::once;

/// Keeps data and state for rendering.
pub struct TableState {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub columns: Vec<ColFormat>,
    pub terminal_size: CharCoord,
    pub cur_pos: TableCoord,
    pub offsets: TableCoord,
    pub command_buffer: Vec<char>,
}

// Factory methods
impl TableState {
    pub fn new(header: Vec<String>, rows: Vec<Vec<String>>, terminal_size: CharCoord) -> Self {
        let col_widths =
            compute_col_widths(once(&header).chain((&rows).iter()), 2, terminal_size.x);
        let columns = col_widths
            .iter()
            .scan(0, |acc, &width| {
                let index = *acc;
                *acc += width;
                Some(ColFormat { width, index })
            })
            .collect();
        let width = terminal_size.x;
        TableState {
            header,
            rows,
            columns,
            terminal_size,
            cur_pos: Default::default(),
            offsets: Default::default(),
            command_buffer: Vec::with_capacity(width),
        }
    }
}

/// Table cell-based coordinates (columns and rows).
#[derive(Debug, Default)]
pub struct TableCoord {
    pub col: usize,
    pub row: usize,
}

/// Character-based coordinates in x and y direction.
#[derive(Debug, Default)]
pub struct CharCoord {
    pub x: usize,
    pub y: usize,
}

/// Formatting information about a column: width and index in characters.
#[derive(Debug, Default)]
pub struct ColFormat {
    pub width: usize,
    pub index: usize,
}

// Implement some helper methods for accessing state.
impl TableState {
    pub fn x_offset(&self) -> usize {
        self.columns[self.offsets.col].index
    }

    pub fn displayable_data_rows(&self) -> usize {
        // need to subtract the header
        self.terminal_size.y - 1
    }

    // Is the final data row visible in the current window?
    pub fn final_row_visible(&self) -> bool {
        self.offsets.row + self.displayable_data_rows() >= self.rows.len()
    }

    // Is the first data row visible in the current window?
    pub fn first_row_visible(&self) -> bool {
        self.offsets.row == 0
    }

    // Is the last data column visible in the current window?
    pub fn last_col_visible(&self) -> bool {
        let last_col = &self.columns[self.columns.len() - 1];
        last_col.index + last_col.width <= self.x_offset() + self.terminal_size.x
    }

    // Is the current row at the bottom of the displayed window?
    pub fn is_bottom(&self) -> bool {
        let bottom_row = min(self.displayable_data_rows(), self.rows.len());
        self.cur_pos.row == bottom_row
    }

    // Absolute index of current column
    pub fn current_column(&self) -> usize {
        self.offsets.col + self.cur_pos.col
    }

    // Absolute index of current row
    pub fn current_row(&self) -> usize {
        self.offsets.row + self.cur_pos.row
    }
}

fn compare_str(a: &str, b: &str) -> Ordering {
    a.cmp(b)
}

fn compare_int(a: &str, b: &str) -> Ordering {
    let a: usize = a.parse().unwrap();
    let b: usize = b.parse().unwrap();
    a.cmp(&b)
}

// Implement user actions. Each methods returns a RenderingAction.
impl TableState {
    pub fn ascending(&mut self, col: usize) -> RenderingAction {
        let comp = if col == 0 { compare_int } else { compare_str };
        self.rows.sort_by(|r1, r2| comp(&r1[col], &r2[col]));
        RenderingAction::Rerender
    }

    pub fn descending(&mut self, col: usize) -> RenderingAction {
        let comp = if col == 0 { compare_int } else { compare_str };
        self.rows.sort_by(|r1, r2| comp(&r2[col], &r1[col]));
        RenderingAction::Rerender
    }

    pub fn execute_command(&mut self) -> RenderingAction {
        if self.command_buffer.len() > 1 && self.command_buffer[0] == '/' {
            self.search(&self.command_buffer[1..].iter().collect::<String>())
        } else {
            RenderingAction::None
        }
    }

    fn jump_to_row(&mut self, row: usize) {
        // first window position
        if row < self.displayable_data_rows() {
            self.offsets.row = 0;
            self.cur_pos.row = row + 1;
        }
        // last window position
        else if self.rows.len() - row < self.displayable_data_rows() {
            self.offsets.row = self.rows.len() - self.displayable_data_rows();
            self.cur_pos.row = row - self.offsets.row + 1;
        }
        // middle
        else {
            self.offsets.row = row;
            self.cur_pos.row = 1;
        }
    }

    pub fn search(&mut self, pattern: &str) -> RenderingAction {
        let col = self.current_column();
        let cur_row = self.current_row();
        for row in (cur_row..self.rows.len()).chain(0..cur_row) {
            let cell = &self.rows[row][col];
            if cell.contains(pattern) {
                self.jump_to_row(row);
                break;
            }
        }
        RenderingAction::Rerender
    }

    pub fn move_down(&mut self) -> RenderingAction {
        if self.is_bottom() {
            if !self.final_row_visible() {
                self.offsets.row += 1;
                return RenderingAction::Rerender;
            }
        } else {
            self.cur_pos.row += 1;
            return RenderingAction::MoveCursor;
        };
        RenderingAction::None
    }

    pub fn move_page_down(&mut self) -> RenderingAction {
        // from the header, we jump to the first data row
        if self.cur_pos.row == 0 {
            self.cur_pos.row = 1;
            RenderingAction::MoveCursor
        }
        // the final row is not yet visible, we need to shift the window to
        else if !self.final_row_visible() {
            self.offsets.row = min(
                // the last window position or
                self.rows.len() - self.displayable_data_rows(),
                // to the next position, making the current last row the first
                self.offsets.row + (self.displayable_data_rows() - 1),
            );
            RenderingAction::Rerender
        }
        // the final row is already within our window
        else if self.cur_pos.row != self.displayable_data_rows() {
            self.cur_pos.row = self.displayable_data_rows();
            RenderingAction::MoveCursor
        } else {
            RenderingAction::None
        }
    }

    pub fn move_up(&mut self) -> RenderingAction {
        if self.cur_pos.row == 1 {
            if !self.first_row_visible() {
                self.offsets.row -= 1;
                return RenderingAction::Rerender;
            } else {
                self.cur_pos.row -= 1;
                return RenderingAction::MoveCursor;
            }
        } else if self.cur_pos.row != 0 {
            self.cur_pos.row -= 1;
            return RenderingAction::MoveCursor;
        };
        RenderingAction::None
    }

    pub fn move_page_up(&mut self) -> RenderingAction {
        if !self.first_row_visible() {
            let new_row = self
                .offsets
                .row
                .wrapping_sub(self.displayable_data_rows() - 1);
            self.offsets.row = if new_row < self.offsets.row {
                new_row
            } else {
                0
            };
            RenderingAction::Rerender
        } else if self.cur_pos.row != 0 {
            self.cur_pos.row = 0;
            RenderingAction::MoveCursor
        } else {
            RenderingAction::None
        }
    }

    pub fn move_home(&mut self) -> RenderingAction {
        self.offsets.row = 0;
        self.cur_pos.row = 0;
        RenderingAction::Rerender
    }

    pub fn move_end(&mut self) -> RenderingAction {
        // all data rows fit into one window
        if self.rows.len() <= self.displayable_data_rows() {
            self.cur_pos.row = self.rows.len();
        }
        // move window to last position and cursor to last row
        else {
            self.offsets.row = self.rows.len() - self.displayable_data_rows();
            self.cur_pos.row = self.terminal_size.y - 1;
        }
        RenderingAction::Rerender
    }

    pub fn move_right(&mut self) -> RenderingAction {
        // We are already in the last column
        if self.current_column() == self.columns.len() - 1 {
            return RenderingAction::None;
        } else {
            self.cur_pos.col += 1;
            let cur_column = self.current_column();
            let new_col = &self.columns[cur_column];
            let new_col_end = new_col.index + new_col.width;
            // The new column is completely within the displayed window
            if new_col_end - self.columns[self.offsets.col].index <= self.terminal_size.x {
                RenderingAction::MoveCursor
            }
            // The new column is (at least partially) outside of the displayed window
            else {
                // Find the first column offset for which the next column fits into the displayed window
                for i in self.offsets.col..(cur_column + 1) {
                    if new_col_end - self.columns[i].index <= self.terminal_size.x {
                        self.cur_pos.col -= i - self.offsets.col;
                        self.offsets.col = i;
                        break;
                    }
                }
                RenderingAction::Rerender
            }
        }
    }

    pub fn move_left(&mut self) -> RenderingAction {
        if self.cur_pos.col == 0 {
            if self.offsets.col != 0 {
                self.offsets.col -= 1;
                return RenderingAction::Rerender;
            }
        } else {
            self.cur_pos.col -= 1;
            return RenderingAction::MoveCursor;
        }
        RenderingAction::None
    }

    pub fn move_start_of_line(&mut self) -> RenderingAction {
        self.cur_pos.col = 0;
        if self.offsets.col == 0 {
            return RenderingAction::MoveCursor;
        }
        self.offsets.col = 0;
        RenderingAction::Rerender
    }

    pub fn move_end_of_line(&mut self) -> RenderingAction {
        let last_col = &self.columns[self.columns.len() - 1];
        let complete_width = last_col.index + last_col.width;
        for (i, col) in self.columns.iter().enumerate() {
            if complete_width - col.index <= self.terminal_size.x {
                self.offsets.col = i;
                self.cur_pos.col = self.columns.len() - i - 1;
                break;
            }
        }
        RenderingAction::Rerender
    }
}

fn compute_col_widths<'a, I>(mut rows: I, padding: usize, window_width: usize) -> Vec<usize>
where
    I: Iterator<Item = &'a Vec<String>>,
{
    let mut widths: Vec<usize> = match rows.next() {
        Some(header) => header.iter().map(|value| value.chars().count()).collect(),
        None => return vec![],
    };
    for row in rows {
        for (i, value) in row.iter().enumerate() {
            let length = value.chars().count();
            if length > widths[i] {
                widths[i] = length;
            }
        }
    }
    // truncate to window width and add padding
    for w in &mut widths {
        *w += padding;
        if *w > window_width {
            *w -= *w - window_width;
        }
    }
    return widths;
}
