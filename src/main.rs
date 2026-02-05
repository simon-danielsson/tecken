use std::{
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::MoveTo,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

mod controls;

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

struct Tecken {
    // program
    sout: Stdout,
    columns: u16,
    rows: u16,
    state: State,

    // logic
    fps: Duration,
    text_entry_buff: String,
}

impl Tecken {
    fn new(sout: Stdout) -> Self {
        Self {
            // program
            sout,
            columns: 0,
            rows: 0,
            // logic
            fps: get_fps(FPS),
            text_entry_buff: String::new(),
            // signals
            state: State::Main,
        }
    }

    fn main_loop(&mut self) -> io::Result<()> {
        self.clear_screen()?;
        self.sout.queue(MoveTo(0, 0))?;
        self.sout.write(self.text_entry_buff.as_bytes())?;
        Ok(())
    }

    fn setup(&mut self) -> io::Result<()> {
        self.sout.execute(EnterAlternateScreen)?;
        (self.columns, self.rows) = terminal::size()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        Ok(())
    }

    fn quit_cleanup(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        self.sout.execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn clear_screen(&mut self) -> io::Result<()> {
        self.sout.queue(Clear(ClearType::All))?;
        Ok(())
    }
}

fn get_fps(fps: f64) -> Duration {
    Duration::from_secs_f64(1.0 / fps)
}
