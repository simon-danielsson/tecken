use std::time::{Duration, Instant};

pub struct StopWatch {
    pub start: Option<Instant>,
    pub total: Duration,
    pub is_active: bool,
}

impl StopWatch {
    pub fn new() -> Self {
        Self {
            start: None,
            total: Duration::ZERO,
            is_active: false,
        }
    }

    pub fn start(&mut self) {
        if !self.is_active {
            self.total = Duration::ZERO;
            self.start = Some(Instant::now());
            self.is_active = true;
        }
    }

    pub fn stop(&mut self) {
        if self.is_active {
            if let Some(s) = self.start.take() {
                self.total += s.elapsed();
            }
            self.is_active = false;
        }
    }

    pub fn elapsed(&self) -> String {
        let duration = match (self.is_active, self.start) {
            (true, Some(s)) => self.total + s.elapsed(),
            _ => self.total,
        };
        let second = format!("{:02}", duration.as_secs() % 60);
        let minute = format!("{:02}", duration.as_secs() / 60);
        format!("{minute}:{second}")
    }

    #[allow(unused)]
    pub fn total(&self) -> u64 {
        let duration = self.total;
        duration.as_secs()
    }

    pub fn reset(&mut self) {
        self.start = None;
        self.is_active = false;
    }
}
