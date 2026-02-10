use std::{
    collections::HashSet,
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use rand::seq::IndexedMutRandom;

mod controls;
mod stopwatch;
mod utils;

const WORDS: &str = include_str!("words.txt");
const ERROR_BG: Color = Color::Red;
const ERROR_FG: Color = Color::White;

const SENT_LEN: i32 = 10;

const USER_ENTRY_HL: Color = Color::Blue;
const FPS: f64 = 100.0;

fn main() -> io::Result<()> {
    let stdout = stdout();

    let mut t = Tecken::new(stdout);

    t.setup()?;

    while t.state != State::Quit {
        t.controls()?;
        if t.first_char_typed {
            t.stopwatch.start();
        }
        t.main_loop()?;
        t.sout.flush()?;
        thread::sleep(t.fps);
    }

    t.quit_cleanup()?;

    // if user exits prematurely, don't print results
    if t.text_entry_buff.chars().count() == t.current_sentence.chars().count() {
        t.print_results();
    }
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
    stopwatch: stopwatch::StopWatch,
    // signal to start stopwatch
    first_char_typed: bool,

    word_pool: Vec<String>,
    current_sentence: String,
    text_entry_buff: String,
    user_typing_errors: i32,
    invalid_letters_col_pos: HashSet<u16>,
}

impl Tecken {
    fn new(sout: Stdout) -> Self {
        Self {
            sout,
            columns: 0,
            rows: 0,
            state: State::Main,
            fps: utils::get_fps(FPS),
            stopwatch: stopwatch::StopWatch::new(),
            first_char_typed: false,
            word_pool: Vec::new(),
            current_sentence: String::new(),
            text_entry_buff: String::new(),
            user_typing_errors: 0,
            invalid_letters_col_pos: HashSet::new(),
        }
    }

    /// WPM: (words - errors) / minutes
    /// Accuracy (%): 1 - (errors / total characters)
    /// Time (sec)
    fn print_results(&mut self) {
        let total_time_sec = self.stopwatch.total() as f64;
        let minutes = total_time_sec / 60.0;

        let total_words = self.current_sentence.split_whitespace().count() as f64;
        let errors = self.user_typing_errors as f64;
        let total_chars = self.current_sentence.chars().count() as f64;

        let wpm = (total_words - errors).max(0.0) / minutes;
        let accuracy = (1.0 - (errors / total_chars)) * 100.0;

        println!("WPM:        {:.1}", wpm);
        println!("Accuracy:   {:.2}%", accuracy);
        println!("Time:       {:.1} sec", total_time_sec);
    }

    fn gen_new_sentence(&mut self) {
        let mut rng = rand::rng();
        let mut sentence = String::new();
        for _ in 0..SENT_LEN {
            if let Some(word) = self.word_pool.choose_mut(&mut rng) {
                sentence.push_str(word);
                sentence.push(' ');
            }
        }
        sentence.pop();
        self.current_sentence = sentence;
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

        let mut new_invalids = HashSet::new();

        for (i, (e_ch, u_ch)) in exercise_chars.iter().zip(user_chars.iter()).enumerate() {
            if e_ch != u_ch {
                new_invalids.insert(i as u16);
            }
        }

        for _ in new_invalids.difference(&self.invalid_letters_col_pos) {
            self.user_typing_errors += 1;
        }

        self.invalid_letters_col_pos = new_invalids;
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

    fn write_metadata(&mut self) -> io::Result<()> {
        self.sout.queue(MoveTo(0, 1))?;
        let s = format!(
            "Time: {} seconds, Errors: {}",
            self.stopwatch.elapsed(),
            self.user_typing_errors
        );
        self.sout.write(s.as_bytes())?;
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

        self.write_metadata()?;

        // if sentence is finished, generate new one
        // if self.text_entry_buff.chars().count() == self.current_sentence.chars().count() {
        //     self.text_entry_buff.clear();
        //     self.current_sentence.clear();
        // }

        // if sentence is finished, exit program
        if self.first_char_typed {
            if self.text_entry_buff.chars().count()
            == self.current_sentence.chars().count()
            {
                self.stopwatch.stop();
                self.state = State::Quit;
            }
        } else {
        }
        Ok(())
    }
}
