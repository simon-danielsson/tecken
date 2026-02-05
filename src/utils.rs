use std::{io, time::Duration};

use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

use crate::Tecken;

impl Tecken {
    pub fn setup(&mut self) -> io::Result<()> {
        self.sout.execute(EnterAlternateScreen)?;
        (self.columns, self.rows) = terminal::size()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        Ok(())
    }

    pub fn quit_cleanup(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        self.sout.execute(LeaveAlternateScreen)?;
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
