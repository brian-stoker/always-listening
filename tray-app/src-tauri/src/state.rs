use std::sync::Mutex;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    VoiceToClaude,
    Dictation,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AppState {
    Idle,
    Active(Mode),
    Recording(Mode),
}

impl AppState {
    pub fn is_idle(&self) -> bool {
        matches!(self, AppState::Idle)
    }

    pub fn active_mode(&self) -> Option<Mode> {
        match self {
            AppState::Active(m) | AppState::Recording(m) => Some(*m),
            AppState::Idle => None,
        }
    }
}

pub struct AppStateManager {
    pub state: Mutex<AppState>,
}

impl AppStateManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(AppState::Idle),
        }
    }

    /// Toggle a mode. If the mode is already active, deactivate it (go idle).
    /// If a different mode is active, switch to the new mode.
    pub fn toggle_mode(&self, mode: Mode) -> AppState {
        let mut state = self.state.lock().unwrap();
        let new_state = match *state {
            AppState::Idle => AppState::Active(mode),
            AppState::Active(current) | AppState::Recording(current) => {
                if current == mode {
                    AppState::Idle
                } else {
                    AppState::Active(mode)
                }
            }
        };
        *state = new_state;
        new_state
    }

    pub fn get_state(&self) -> AppState {
        *self.state.lock().unwrap()
    }
}
