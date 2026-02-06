use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppState {
    Idle,
    Active(String),
    Recording(String),
    AgentSpeaking(String),
}

impl AppState {
    #[allow(dead_code)]
    pub fn is_idle(&self) -> bool {
        matches!(self, AppState::Idle)
    }

    pub fn active_agent(&self) -> Option<&str> {
        match self {
            AppState::Active(id) | AppState::Recording(id) | AppState::AgentSpeaking(id) => {
                Some(id)
            }
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

    /// Toggle an agent. If the same agent is active, deactivate (go idle).
    /// If a different agent is active, switch to the new one.
    pub fn toggle_agent(&self, agent_id: &str) -> AppState {
        let mut state = self.state.lock().unwrap();
        let new_state = match &*state {
            AppState::Idle => AppState::Active(agent_id.to_string()),
            AppState::Active(current)
            | AppState::Recording(current)
            | AppState::AgentSpeaking(current) => {
                if current == agent_id {
                    AppState::Idle
                } else {
                    AppState::Active(agent_id.to_string())
                }
            }
        };
        *state = new_state.clone();
        new_state
    }

    /// Set agent directly. None = Idle.
    pub fn set_agent(&self, agent_id: Option<&str>) -> AppState {
        let mut state = self.state.lock().unwrap();
        let new_state = match agent_id {
            Some(id) => AppState::Active(id.to_string()),
            None => AppState::Idle,
        };
        *state = new_state.clone();
        new_state
    }

    pub fn get_state(&self) -> AppState {
        self.state.lock().unwrap().clone()
    }

    /// Transition to Recording state (user is speaking)
    pub fn set_recording(&self) -> AppState {
        let mut state = self.state.lock().unwrap();
        if let Some(id) = state.active_agent().map(|s| s.to_string()) {
            *state = AppState::Recording(id);
        }
        state.clone()
    }

    /// Transition to AgentSpeaking state
    pub fn set_agent_speaking(&self) -> AppState {
        let mut state = self.state.lock().unwrap();
        if let Some(id) = state.active_agent().map(|s| s.to_string()) {
            *state = AppState::AgentSpeaking(id);
        }
        state.clone()
    }

    /// Transition back to Active state (done recording/speaking)
    pub fn set_active(&self) -> AppState {
        let mut state = self.state.lock().unwrap();
        if let Some(id) = state.active_agent().map(|s| s.to_string()) {
            *state = AppState::Active(id);
        }
        state.clone()
    }
}
