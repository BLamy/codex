#[derive(Debug, Clone)]
pub struct SleepInhibitor {
    turn_running: bool,
}

impl SleepInhibitor {
    pub fn new(_enabled: bool) -> Self {
        Self {
            turn_running: false,
        }
    }

    pub fn set_turn_running(&mut self, turn_running: bool) {
        self.turn_running = turn_running;
    }

    pub fn is_turn_running(&self) -> bool {
        self.turn_running
    }
}
