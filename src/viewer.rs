//! Handles user input and uses table state and renderer to update terminal.
use crate::renderer::{RenderingAction, TableRenderer};
use crate::state::TableState;
use crate::termion::input::TermRead;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{stdout, Write};
use termion::event::Key;
use termion::raw::IntoRawMode;

pub struct TableViewer<T: TableRenderer> {
    state: TableState,
    renderer: T,
    mode: Mode,
}

enum Mode {
    Normal,
    Command,
}

impl<T: TableRenderer> TableViewer<T> {
    pub fn new(renderer: T, header: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        let state = TableState::new(header, rows, renderer.window_size());
        let mode = Mode::Normal;
        TableViewer {
            state,
            renderer,
            mode,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout().into_raw_mode().unwrap();
        let stdin = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
        if let Some(value) = self
            .renderer
            .render(&self.state, &RenderingAction::Rerender)
        {
            print!("{}", value);
            stdout.flush()?;
        }
        let mut prev_key: Key = Key::Home;
        for c in stdin.keys() {
            let key = c.unwrap();
            let action = match self.mode {
                Mode::Normal => match key {
                    // Quit app
                    Key::Char('q') | Key::Ctrl('q') | Key::Ctrl('x') | Key::Ctrl('c') => {
                        RenderingAction::Reset
                    }
                    // Sort by column: ascending or descending
                    Key::Char('a') => self.state.ascending(self.state.current_column()),
                    Key::Char('d') => self.state.descending(self.state.current_column()),
                    Key::Char('o') => self.state.ascending(0),
                    // Navigation
                    Key::Down | Key::Char('j') => self.state.move_down(),
                    Key::Up | Key::Char('k') => self.state.move_up(),
                    Key::PageDown => self.state.move_page_down(),
                    Key::PageUp => self.state.move_page_up(),
                    Key::Home => self.state.move_home(),
                    Key::Char('g') if prev_key == Key::Char('g') => self.state.move_home(),
                    Key::End | Key::Char('G') => self.state.move_end(),
                    Key::Right | Key::Char('l') => self.state.move_right(),
                    Key::Left | Key::Char('h') => self.state.move_left(),
                    Key::Char('0') => self.state.move_start_of_line(),
                    Key::Char('$') => self.state.move_end_of_line(),
                    // Switch to command mode
                    Key::Char('/') => {
                        self.mode = Mode::Command;
                        self.state.command_buffer.clear();
                        self.state.command_buffer.push('/');
                        RenderingAction::Command
                    }
                    // Repeat last command
                    Key::Char(' ') => self.state.execute_command(),
                    _ => RenderingAction::None,
                },
                Mode::Command => match key {
                    // Quit app
                    Key::Ctrl('q') | Key::Ctrl('x') | Key::Ctrl('c') => RenderingAction::Reset,
                    // Execute command
                    Key::Char('\n') => {
                        self.mode = Mode::Normal;
                        if self.state.command_buffer.len() <= 1 {
                            RenderingAction::Rerender
                        } else {
                            self.state.execute_command()
                        }
                    }
                    // Enter command character
                    Key::Char(c) => {
                        self.state.command_buffer.push(c);
                        RenderingAction::Command
                    }
                    // Delete command character
                    Key::Backspace => {
                        self.state.command_buffer.pop();
                        if self.state.command_buffer.is_empty() {
                            self.mode = Mode::Normal;
                            RenderingAction::Rerender
                        } else {
                            RenderingAction::Command
                        }
                    }
                    // Switch to normal mode
                    Key::Esc => {
                        self.mode = Mode::Normal;
                        self.state.command_buffer.clear();
                        RenderingAction::Rerender
                    }
                    _ => RenderingAction::None,
                },
            };
            if let Some(value) = self.renderer.render(&self.state, &action) {
                print!("{}", value);
                stdout.flush()?;
            }
            if let RenderingAction::Reset = action {
                break;
            }
            prev_key = key;
        }
        Ok(())
    }
}
