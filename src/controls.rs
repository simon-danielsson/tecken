use std::time::Duration;

use crate::{State, Tecken};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, poll};

impl Tecken {
    pub fn controls(&mut self) -> std::io::Result<()> {
        if poll(Duration::ZERO)? {
            match self.state {
                State::Main => {
                    if let Event::Key(KeyEvent {
                        code, modifiers, ..
                    }) = event::read()?
                    {
                        match (code, modifiers) {
                            // quit
                            (KeyCode::Esc, _) => {
                                self.state = State::Quit;
                            }

                            (
                                KeyCode::Char('c'),
                                KeyModifiers::CONTROL,
                            ) => {
                                self.state = State::Quit;
                            }

                            // backspace
                            (KeyCode::Backspace, KeyModifiers::ALT) => {
                                _ = self.text_entry_buff.pop();
                                let buff = self
                                    .text_entry_buff
                                    .clone();

                                let mut count = 0;
                                let mut i = 0;
                                let mut space_found = false;
                                for c in buff.chars().rev() {
                                    if !space_found {
                                        if c == ' ' {
                                            space_found = true;
                                            i = buff.chars()
                                                .count()
                                                - count;
                                        }
                                    }
                                    count += 1;
                                }

                                if !space_found {
                                    self.text_entry_buff
                                        .clear();
                                } else {
                                    self.text_entry_buff
                                        .truncate(i);
                                }
                            }

                            (KeyCode::Backspace, _) => {
                                self.text_entry_buff.pop();
                            }

                            // type characters
                            (KeyCode::Char(c), _) => {
                                if !self.first_char_typed {
                                    self.first_char_typed =
                                        true;
                                }
                                self.text_entry_buff.push(c);
                            }

                            _ => {}
                        }
                    }
                }

                State::Quit => {}
            }
        }
        Ok(())
    }
}
