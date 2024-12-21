use std::io;

use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    Terminal,
};

use crate::app::{App, CurrentScreen, CurrentlyEditing};

impl App<'_> {
    pub fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<bool> {
        loop {
            self.render_json();
            terminal.draw(|f| self.ui(f))?;
            if let Some(res) = self.handle_event() {
                return Ok(res);
            }
        }
    }

    pub fn handle_event(&mut self) -> Option<bool> {
        let event = match event::read() {
            Ok(event) => event,
            Err(e) => {
                eprintln!("Error: {:?}", e);
                return None;
            }
        };
        if let Event::Key(key) = event {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                return None;
            }
            match self.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('e') => {
                        self.current_screen = CurrentScreen::Editing;
                        self.currently_editing = Some(CurrentlyEditing::Key);
                    }
                    KeyCode::Char('q') => {
                        self.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },

                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Some(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Some(false);
                    }
                    _ => {}
                },

                CurrentScreen::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => {
                        if let Some(editing) = &self.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    self.currently_editing = Some(CurrentlyEditing::Value);
                                }
                                CurrentlyEditing::Value => {
                                    self.save_key_value();
                                    self.current_screen = CurrentScreen::Main;
                                }
                            }
                        }
                    }

                    KeyCode::Backspace => {
                        if let Some(editing) = &self.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    self.key_input.pop();
                                }
                                CurrentlyEditing::Value => {
                                    self.value_input.pop();
                                }
                            }
                        }
                    }

                    KeyCode::Esc => {
                        self.current_screen = CurrentScreen::Main;
                        self.currently_editing = None;
                    }

                    KeyCode::Tab => {
                        self.toggle_editing();
                    }

                    KeyCode::Char(value) => {
                        if let Some(editing) = &self.currently_editing {
                            match editing {
                                CurrentlyEditing::Key => {
                                    self.key_input.push(value);
                                }
                                CurrentlyEditing::Value => {
                                    self.value_input.push(value);
                                }
                            }
                        }
                    }

                    _ => {}
                },
                _ => {}
            }
        }
        None
    }
}
