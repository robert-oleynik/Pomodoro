use std::time::{Duration, SystemTime};

pub mod todo;

pub struct State {
    pub state: Pomodoro,
    pub until: SystemTime,
    pub notified: bool,
    pub round: u64,
}

#[derive(Clone, Copy)]
pub enum Pomodoro {
    Working,
    Pause,
}

impl Default for State {
    fn default() -> Self {
        Self {
            state: Pomodoro::Pause,
            until: SystemTime::now(),
            notified: true,
            round: 0,
        }
    }
}

impl State {
    pub fn next(&mut self, duration: Duration) {
        if let Pomodoro::Pause = self.state {
            self.round += 1;
        }
        self.state = match self.state {
            Pomodoro::Working => Pomodoro::Pause,
            Pomodoro::Pause => Pomodoro::Working,
        };
        self.until = SystemTime::now().checked_add(duration).unwrap();
        self.notified = false;
    }
}
