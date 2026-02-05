use std::{
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, ResetColor, SetForegroundColor},
};

mod controls;
mod utils;

#[allow(unused)]
const ERROR_BG: Color = Color::Blue;
#[allow(unused)]
const ERROR_FG: Color = Color::Black;

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
        }
    }

    /// takes a string and returns the column the string would
    /// have to be written from to make it look centered
    fn col_c_with_offs(&mut self, s: &str) -> u16 {
        let length = (s.trim().chars().count()) / 3;
        (self.columns / 2).saturating_sub(length as u16)
    }

    fn write_word(&mut self, s: String) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(Color::Reset))?;
        // self.sout.queue(SetForegroundColor(Color::Black))?;
        let col = self.col_c_with_offs(&s);
        self.sout.queue(MoveTo(col, self.rows / 2))?;
        self.sout.write(s.as_bytes())?;
        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn write_user_entry(&mut self) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(USER_ENTRY_HL))?;
        let input = self.text_entry_buff.clone();
        let col = self.col_c_with_offs(&input);
        self.sout.queue(MoveTo(col, self.rows / 2))?;
        self.sout.write(self.text_entry_buff.as_bytes())?;
        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn main_loop(&mut self) -> io::Result<()> {
        self.clear_screen()?;
        self.write_word(self.current_sentence.clone())?;
        self.write_user_entry()?;
        self.sout.queue(MoveTo(0, 0))?;
        self.sout.write(self.text_entry_buff.as_bytes())?;
        Ok(())
    }
}
