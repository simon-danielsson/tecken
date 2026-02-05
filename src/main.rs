use std::{
    io::{self, Stdout, Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal::{
        self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
        disable_raw_mode, enable_raw_mode,
    },
};

const FPS: f64 = 30.0;

fn main() -> io::Result<()> {
    let stdout = stdout();

    let mut t = Tecken::new(stdout);

    t.setup()?;

    while !t.sign_end {
        t.main_loop()?;
        t.sout.flush()?;
        thread::sleep(t.fps);
    }

    t.exit_cleanup()?;
    Ok(())
}

struct Tecken {
    // program
    sout: Stdout,
    columns: u16,
    rows: u16,

    // logic
    fps: Duration,

    // signals
    sign_end: bool,
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
            // signals
            sign_end: false,
        }
    }

    fn main_loop(&mut self) -> io::Result<()> {
        thread::sleep(Duration::from_secs(5));
        self.sign_end = true;
        Ok(())
    }

    fn setup(&mut self) -> io::Result<()> {
        self.sout.execute(EnterAlternateScreen)?;
        (self.columns, self.rows) = terminal::size()?;
        enable_raw_mode()?;
        self.clear_screen()?;
        Ok(())
    }

    fn exit_cleanup(&mut self) -> io::Result<()> {
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
