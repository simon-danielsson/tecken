use std::{
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
};

mod controls;
mod utils;

const ERROR_BG: Color = Color::Red;
const ERROR_FG: Color = Color::White;

const USER_ENTRY_HL: Color = Color::Blue;
const FPS: f64 = 100.0;

fn main() -> io::Result<()> {
    let stdout = stdout();

    let mut t = Tecken::new(stdout);

    t.setup()?;

    while t.state != State::Quit {
        t.controls()?;
        t.main_loop()?;
        t.sout.flush()?;
        thread::sleep(t.fps);
    }

    t.quit_cleanup()?;
    Ok(())
}

#[derive(PartialEq)]
enum State {
    Main,
    Quit,
}

#[allow(unused)]
#[derive(Clone, PartialEq)]
struct Pos {
    col: u16,
    row: u16,
}
impl Pos {
    #[allow(unused)]
    fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

struct Tecken {
    sout: Stdout,
    columns: u16,
    rows: u16,
    state: State,
    fps: Duration,

    current_sentence: String,
    text_entry_buff: String,
    user_typing_errors: i32,
    invalid_letters_col_pos: Vec<u16>,
}

impl Tecken {
    fn new(sout: Stdout) -> Self {
        Self {
            sout,
            columns: 0,
            rows: 0,
            state: State::Main,
            fps: utils::get_fps(FPS),

            current_sentence: String::from("this is a sentence"),
            text_entry_buff: String::new(),
            user_typing_errors: 0,
            invalid_letters_col_pos: Vec::new(),
        }
    }

    fn write_word(&mut self, s: String) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(Color::Blue))?;
        self.sout.queue(SetBackgroundColor(Color::Reset))?;
        self.sout.queue(MoveTo(0, 0))?;
        self.sout.write(s.as_bytes())?;
        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn write_user_entry(&mut self) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(Color::Black))?;
        self.sout.queue(SetBackgroundColor(USER_ENTRY_HL))?;
        self.sout.queue(MoveTo(0, 0))?;
        self.sout.write(self.text_entry_buff.as_bytes())?;
        self.sout.queue(ResetColor)?;
        Ok(())
    }

    /// loop through each character of usr ip and
    /// validate against exercise
    fn validation(&mut self) -> io::Result<()> {
        let exercise_chars: Vec<char> = self.current_sentence.chars().collect();
        let user_chars: Vec<char> = self.text_entry_buff.chars().collect();
        self.invalid_letters_col_pos.clear();
        self.user_typing_errors = 0;
        for (i, (e_ch, u_ch)) in exercise_chars.iter().zip(user_chars.iter()).enumerate() {
            if e_ch != u_ch {
                self.user_typing_errors += 1;
                self.invalid_letters_col_pos.push(i as u16);
            }
        }
        Ok(())
    }

    fn write_errors(&mut self) -> io::Result<()> {
        let current_row = 0;

        let user_chars: Vec<char> = self.text_entry_buff.chars().collect();

        for &col in &self.invalid_letters_col_pos {
            let idx = col as usize;

            if let Some(&ch) = user_chars.get(idx) {
                self.sout.queue(MoveTo(col, current_row))?;
                self.sout.queue(SetBackgroundColor(ERROR_BG))?;
                self.sout.queue(SetForegroundColor(ERROR_FG))?;
                self.sout.write_all(ch.to_string().as_bytes())?;
            }
        }
        self.sout.queue(SetBackgroundColor(Color::Reset))?;
        self.sout.queue(SetForegroundColor(Color::Reset))?;

        Ok(())
    }

    fn main_loop(&mut self) -> io::Result<()> {
        self.clear_screen()?;
        // write exercise word
        self.write_word(self.current_sentence.clone())?;
        // write user type
        self.write_user_entry()?;
        // check for errors in input
        self.validation()?;
        self.write_errors()?;
        Ok(())
    }
}
