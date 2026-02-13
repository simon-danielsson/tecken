use std::{io, time::Duration};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

use crate::{Tecken, WORDS, stopwatch::StopWatch};

impl Tecken {
    pub fn setup(&mut self) -> io::Result<()> {
        self.sout.execute(EnterAlternateScreen)?;
        (self.columns, self.rows) = terminal::size()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        self.gen_word_pool();
        self.sout.queue(cursor::SavePosition)?;
        self.sout.queue(cursor::Hide)?;
        self.line_length = self.f_word_quantity / 2;
        self.gen_new_sentence();
        Ok(())
    }

    pub fn endless_mode_next_sentence(&mut self) -> io::Result<()> {
        // state reset
        self.input_registered = false;
        self.first_char_typed = false;
        self.invalid_letters_col_pos.clear();
        self.text_entry_buff.clear();
        self.exercise_text_text.clear();
        self.exercise_text_lines.clear();
        self.user_typing_errors = 0;
        self.stopwatch.stop();
        self.stopwatch.reset();
        self.stopwatch = StopWatch::new();

        // new setup
        self.clear_screen()?;
        self.gen_word_pool();
        self.gen_new_sentence();
        Ok(())
    }

    pub fn exercise_finished(&mut self) -> bool {
        self.text_entry_buff.chars().count() == self.exercise_text_text.chars().count()
    }

    /// calculate the column pos required so that a line can be centered
    pub fn center_line(&mut self, line: String) -> u16 {
        let center_of_vp = self.columns / 2;
        let line_length = line.chars().count();
        let centered = center_of_vp as usize - (line_length / 2);
        return centered as u16;
    }

    pub fn gen_word_pool(&mut self) {
        self.word_pool = WORDS.split_whitespace().map(String::from).collect();
    }

    pub fn quit_cleanup(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        self.sout.execute(LeaveAlternateScreen)?;
        self.sout.queue(cursor::RestorePosition)?;
        self.sout.queue(cursor::Show)?;
        Ok(())
    }

    pub fn clear_screen(&mut self) -> io::Result<()> {
        self.sout.queue(Clear(ClearType::All))?;
        Ok(())
    }
}

pub fn get_fps(fps: f64) -> Duration {
    Duration::from_secs_f64(1.0 / fps)
}
