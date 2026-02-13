use std::io;

use crate::{State, Tecken};

impl Tecken {
    pub fn parse_args(&mut self) -> io::Result<()> {
        let mut it = std::env::args().skip(1); // skip program name
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "-w" => {
                    // use next if it exists and parses as i32, else default to 12
                    self.f_word_quantity = it
                        .next()
                        .as_deref()
                        .unwrap_or("12")
                        .parse::<i32>()
                        .unwrap_or(12);
                }
                "help" => {
                    self.state = State::Help;
                    return Ok(());
                }
                "-e" => {
                    self.f_endless_mode = true;
                    self.state = State::Endless;
                }
                _ => {} // "-l" => self.f_language =
                // "-d" => self.f_difficulty = true,
                // other => {
                //     self.opt_dir = PathBuf::from(other);
                //     if self.opt_dir.exists() {
                //         self.use_opt_dir = true;
                //     } else {
                //         eprintln!("Directory doesn't exist!");
                //     }
                //     break; // stop after first positional
                // }
            }
        }
        Ok(())
    }
}
