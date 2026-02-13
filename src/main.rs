use std::{
    collections::HashSet,
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use rand::Rng;

mod arg_parse;
mod controls;
mod stopwatch;
mod subcommands;
mod utils;

// === constants ===

// general

const WORDS: &str = include_str!("static/words.txt");
const FPS: f64 = 120.0;

// colors

const CLR_ERROR_BG: Color = Color::Red;
const CLR_ERROR_FG: Color = Color::Black;
const CLR_EXERCISE_BG: Color = Color::Reset;
const CLR_EXERCISE_FG: Color = Color::Blue;
const CLR_ENTRY_BG: Color = Color::Blue;
const CLR_ENTRY_FG: Color = Color::Black;

// === code ===

fn main() -> io::Result<()> {
    let stdout = stdout();

    let mut t = Tecken::new(stdout);

    t.parse_args()?;

    if t.state == State::Help {
        t.s_help();
        return Ok(());
    }

    t.setup()?;

    while t.state != State::Quit {
        t.controls()?;
        if t.first_char_typed {
            t.stopwatch.start();
        }
        if t.state == State::Main || t.state == State::Endless {
            t.main_loop()?;
        }
        t.sout.flush()?;
        thread::sleep(t.fps);
    }

    t.quit_cleanup()?;

    // if user exits prematurely or is exiting endless mode, don't print results
    if !t.f_endless_mode {
        if t.exercise_finished() {
            t.print_results();
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum State {
    Main,
    Endless,
    Help,
    Quit,
}

#[allow(unused)]
enum BorderType {
    Single,
    Double,
}

struct Rect {
    pos: Pos,
    width: u16,
    height: u16,
    border: BorderType,
}
impl Rect {
    fn new(pos: Pos, width: u16, height: u16, border: BorderType) -> Self {
        Self {
            pos,
            width,
            height,
            border,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Pos {
    col: u16,
    row: u16,
}
impl Pos {
    fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

struct Line {
    /// words
    text: Vec<String>,
    /// col: start of line
    pos: Pos,
}
impl Line {
    fn new(text: Vec<String>, pos: Pos) -> Self {
        Self { text, pos }
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
    // signal to update ui only when input registered
    input_registered: bool,
    word_pool: Vec<String>,
    exercise_text_lines: Vec<Line>,
    exercise_text_text: String,
    text_entry_buff: String,
    user_typing_errors: i32,
    invalid_letters_col_pos: HashSet<u16>,
    line_length: i32,
    // flags & subcommands
    f_word_quantity: i32,
    f_endless_mode: bool,
    f_hide_metadata: bool,
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
            input_registered: false,
            word_pool: Vec::new(),
            exercise_text_text: String::new(),
            exercise_text_lines: Vec::new(),
            text_entry_buff: String::new(),
            user_typing_errors: 0,
            invalid_letters_col_pos: HashSet::new(),
            line_length: 0,
            // flags & subcommands
            f_word_quantity: 12,
            f_endless_mode: false,
            f_hide_metadata: false,
        }
    }

    /// WPM: (words - errors) / minutes
    /// Accuracy (%): 1 - (errors / total characters)
    /// Time (sec)
    fn print_results(&mut self) {
        let total_time_sec = self.stopwatch.total() as f64;
        let minutes = total_time_sec / 60.0;

        let total_words = self.exercise_text_text.split_whitespace().count() as f64;
        let total_chars = self.exercise_text_text.chars().count() as f64;
        let errors = self.user_typing_errors as f64;

        let raw_wpm = (total_words).max(0.0) / minutes;
        let wpm = (total_words - (errors / 2.0)).max(0.0) / minutes;
        let accuracy = (1.0 - (errors / total_chars)) * 100.0;

        println!("Raw WPM:    {:.1}", raw_wpm);
        println!("WPM:        {:.1}", wpm);
        println!("Accuracy:   {:.2}%", accuracy);
        println!("Time:       {:.1} sec", total_time_sec);
    }

    fn gen_new_sentence(&mut self) {
        let mut rng = rand::rng();

        // get words from word pool
        let words: Vec<String> = {
            let mut words = Vec::new();
            while words.len() < self.f_word_quantity as usize {
                let r = rng.random_range(..self.word_pool.len());
                words.push(self.word_pool[r].clone());
            }
            words
        };

        // calc approx. number of lines required (only used for centering text vertically)
        let num_of_lines: i32 = {
            let words_len = words.concat().chars().count();
            let n = words_len as i32 / self.rows as i32;
            n
        };

        let mut exercise_text_text = String::new();
        let center_row = self.rows / 2;
        let starting_row: u16 = center_row - (num_of_lines as u16 / 2);
        let mut lines_added: i32 = 0;
        let max_line_len = self.columns as i32 - (self.columns as i32 / 2);
        let mut it = words.iter();
        let mut init = true;
        while let Some(word) = it.next() {
            let mut line_str = Vec::new();
            let mut line_len: i32 = 0;

            // add inital word
            if init {
                line_str.push(word.to_string());
                line_str.push(" ".to_string());
                line_len += word.chars().count() as i32;
                init = false;
            }

            // create a new line
            while line_len < max_line_len {
                if let Some(val) = it.next().as_deref() {
                    line_str.push(val.to_string());
                    line_str.push(" ".to_string());
                }
                line_len += word.chars().count() as i32 + 1;
            }
            lines_added += 1;

            exercise_text_text.push_str(&line_str.concat());
            line_str.pop();
            let line_str_as_string = line_str.concat();
            let pos = Pos::new(
                self.center_line(line_str_as_string),
                starting_row + lines_added as u16,
            );

            self.exercise_text_lines.push(Line::new(line_str, pos));
        }
        _ = exercise_text_text.pop();
        self.exercise_text_text = exercise_text_text;
    }

    fn w_exercise_text(&mut self) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(CLR_EXERCISE_FG))?;
        self.sout.queue(SetBackgroundColor(CLR_EXERCISE_BG))?;
        for line in self.exercise_text_lines.iter_mut() {
            self.sout.queue(MoveTo(line.pos.col, line.pos.row))?;
            let text = line.text.concat();
            self.sout.write(text.as_bytes())?;
        }
        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn w_user_entry(&mut self) -> io::Result<()> {
        self.sout.queue(SetForegroundColor(CLR_ENTRY_FG))?;
        self.sout.queue(SetBackgroundColor(CLR_ENTRY_BG))?;

        let user_chars: Vec<char> = self.text_entry_buff.chars().collect();
        let mut offset: usize = 0;

        for (i, line) in self.exercise_text_lines.iter().enumerate() {
            let line_len = line.text.concat().chars().count();

            if offset >= user_chars.len() {
                break;
            }

            let end = (offset + line_len).min(user_chars.len());
            let typed_segment: String = user_chars[offset..end].iter().collect();

            self.sout.queue(MoveTo(line.pos.col, line.pos.row))?;
            self.sout.write_all(typed_segment.as_bytes())?;

            offset += line_len;

            // skip space at line change to not add extra offset
            if i != self.exercise_text_lines.len() - 1 {
                offset += 1;
            }
        }

        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn char_idx_to_pos(&self, idx: usize) -> Option<Pos> {
        let mut offset = 0usize;

        for line in &self.exercise_text_lines {
            let line_len = line.text.concat().chars().count();

            if idx < offset + line_len {
                let col = line.pos.col + (idx - offset) as u16;
                let row = line.pos.row;
                return Some(Pos::new(col, row));
            }

            offset += line_len;

            // skip space at line change to not add extra offset
            if line_len != self.exercise_text_lines.len() - 1 {
                offset += 1;
            }
        }

        None
    }

    /// loop through each character of usr ip and
    /// validate against exercise
    fn validation(&mut self) -> io::Result<()> {
        let exercise_chars: Vec<char> = self.exercise_text_text.chars().collect();
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

    fn w_errors(&mut self) -> io::Result<()> {
        let user_chars: Vec<char> = self.text_entry_buff.chars().collect();

        for &idx_u16 in &self.invalid_letters_col_pos {
            let idx = idx_u16 as usize;

            if let (Some(&ch), Some(pos)) =
            (user_chars.get(idx), self.char_idx_to_pos(idx))
            {
                self.sout.queue(MoveTo(pos.col, pos.row))?;
                self.sout.queue(SetBackgroundColor(CLR_ERROR_BG))?;
                self.sout.queue(SetForegroundColor(CLR_ERROR_FG))?;
                self.sout.write_all(ch.to_string().as_bytes())?;
            }
        }

        self.sout.queue(ResetColor)?;
        Ok(())
    }

    fn w_metadata(&mut self) -> io::Result<()> {
        // get the last row of the exercise text to adhere to
        let prev_row: u16 = {
            let last_line = self.exercise_text_lines.last().unwrap();
            last_line.pos.row
        };

        let time_s = format!("Elapsed : {}", self.stopwatch.elapsed());
        let err_s = format!("Errors : {}", self.user_typing_errors);

        let time_col = self.center_line(time_s.clone());
        let err_col = self.center_line(err_s.clone());

        self.sout.queue(MoveTo(time_col, prev_row + 2))?;
        self.sout.write(time_s.as_bytes())?;
        self.sout.queue(MoveTo(err_col, prev_row + 3))?;
        self.sout.write(err_s.as_bytes())?;
        Ok(())
    }

    /// write a box using type Rect
    fn w_rect(&mut self, r: Rect) -> io::Result<()> {
        let b = match r.border {
            BorderType::Single => ['╭', '─', '╮', '│', '╯', '╰'],
            BorderType::Double => ['╔', '═', '╗', '║', '╝', '╚'],
        };

        // if nothing
        if r.width == 0 || r.height == 0 {
            return Ok(());
        }

        let x0 = r.pos.col;
        let y0 = r.pos.row;
        let w = r.width as u16;
        let h = r.height as u16;

        // 1x1: just a corner char (pick top-left)
        if w == 1 && h == 1 {
            self.sout.queue(MoveTo(x0, y0))?;
            self.sout.queue(Print(b[0]))?;
            return Ok(());
        }

        // repeat horizontal segment count times
        let horiz_len = w.saturating_sub(2) as usize;
        let horiz = b[1].to_string().repeat(horiz_len);

        // top row
        self.sout.queue(MoveTo(x0, y0))?;
        if w == 1 {
            self.sout.queue(Print(b[0]))?;
        } else {
            self.sout
                .queue(Print(b[0]))?
                .queue(Print(&horiz))?
                .queue(Print(b[2]))?;
        }

        // verticals
        let mid_rows = h.saturating_sub(2);
        for dy in 0..mid_rows {
            let yy = y0 + 1 + dy;
            self.sout.queue(MoveTo(x0, yy))?.queue(Print(b[3]))?;
            if w > 1 {
                self.sout
                    .queue(MoveTo(x0 + w - 1, yy))?
                    .queue(Print(b[3]))?;
            }
        }

        // bottom row
        if h > 1 {
            let yb = y0 + h - 1;
            self.sout.queue(MoveTo(x0, yb))?;
            if w == 1 {
                self.sout.queue(Print(b[5]))?;
            } else {
                self.sout
                    .queue(Print(b[5]))?
                    .queue(Print(&horiz))?
                    .queue(Print(b[4]))?;
            }
        }
        Ok(())
    }

    fn main_loop(&mut self) -> io::Result<()> {
        if self.input_registered {
            self.clear_screen()?;
        }

        // write surrounding frame ui
        let main_frame = Rect::new(
            Pos { col: 0, row: 0 },
            self.columns,
            self.rows,
            BorderType::Double,
        );
        self.w_rect(main_frame)?;

        self.w_exercise_text()?;
        self.w_user_entry()?;
        self.validation()?;
        self.w_errors()?;

        if !self.f_hide_metadata {
            self.w_metadata()?;
        }

        // if sentence is finished, exit program
        if self.state == State::Main {
            if self.first_char_typed {
                if self.exercise_finished() {
                    self.stopwatch.stop();
                    self.state = State::Quit;
                }
            }
        }
        if self.state == State::Endless {
            if self.first_char_typed {
                if self.exercise_finished() {
                    self.endless_mode_next_sentence()?;
                }
            }
        }
        self.input_registered = false;
        // }
        Ok(())
    }
}
