use std::{io, time::Duration};

use crossterm::{
    ExecutableCommand, QueueableCommand, cursor,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

use crate::{Tecken, WORDS};

impl Tecken {
    pub fn setup(&mut self) -> io::Result<()> {
        self.sout.execute(EnterAlternateScreen)?;
        (self.columns, self.rows) = terminal::size()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        self.gen_word_pool();
        self.sout.queue(cursor::SavePosition)?;
        self.sout.queue(cursor::Hide)?;
        self.gen_new_sentence();
        Ok(())
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
