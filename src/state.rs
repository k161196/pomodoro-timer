use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTimer {
    pub id: String,  // Unique ID for this timer session
    pub label: String,
    pub duration_secs: u32,
    pub session_type: String,  // "Work", "Short Break", "Long Break"
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimerState {
    Idle,
    Working,
    WorkPaused,
    ShortBreak,
    BreakPaused,
    LongBreak,
    LongBreakPaused,
}

impl TimerState {
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            TimerState::Working | TimerState::ShortBreak | TimerState::LongBreak
        )
    }

    pub fn is_work(&self) -> bool {
        matches!(self, TimerState::Working | TimerState::WorkPaused)
    }

    pub fn pause(&self) -> Option<TimerState> {
        match self {
            TimerState::Working => Some(TimerState::WorkPaused),
            TimerState::ShortBreak => Some(TimerState::BreakPaused),
            TimerState::LongBreak => Some(TimerState::LongBreakPaused),
            _ => None,
        }
    }

    pub fn resume(&self) -> Option<TimerState> {
        match self {
            TimerState::WorkPaused => Some(TimerState::Working),
            TimerState::BreakPaused => Some(TimerState::ShortBreak),
            TimerState::LongBreakPaused => Some(TimerState::LongBreak),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            TimerState::Idle => "Ready",
            TimerState::Working | TimerState::WorkPaused => "Work Session",
            TimerState::ShortBreak | TimerState::BreakPaused => "Short Break",
            TimerState::LongBreak | TimerState::LongBreakPaused => "Long Break",
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub current_state: TimerState,
    pub time_remaining_secs: u32,
    pub rest_time_remaining_secs: u32, // Separate timer for rest mode
    pub is_focus_mode: bool,       // true = focus/work, false = rest/break
    pub current_session: u32,      // Current session number (1-4)
    pub completed_sessions: u32,   // Total completed today
    pub last_updated: DateTime<Utc>,
    pub current_id: String,        // Unique ID for current timer session
    pub current_label: String,     // Label for current timer
    pub history: Vec<CompletedTimer>, // History of completed timers
    pub history_index: Option<usize>, // Current index when browsing history (None = current timer)
    #[serde(default)]
    pub show_celebration: bool,    // True when timer just completed (breathing effect)
}

impl SessionInfo {
    pub fn new() -> Self {
        Self {
            current_state: TimerState::Idle,
            time_remaining_secs: 0,
            rest_time_remaining_secs: 0,
            is_focus_mode: true,  // Default to focus mode
            current_session: 1,
            completed_sessions: 0,
            last_updated: Utc::now(),
            current_id: Uuid::new_v4().to_string(),
            current_label: String::new(),
            history: Vec::new(),
            history_index: None,
            show_celebration: false,
        }
    }

    pub fn add_to_history(&mut self, id: String, label: String, duration_secs: u32, session_type: String) {
        self.history.push(CompletedTimer {
            id,
            label,
            duration_secs,
            session_type,
            completed_at: Utc::now(),
        });
        // Keep only last 50 timers
        if self.history.len() > 50 {
            self.history.remove(0);
        }
        // Generate new ID for next session
        self.current_id = Uuid::new_v4().to_string();
    }


    pub fn exit_history(&mut self) {
        self.history_index = None;
    }

    pub fn navigate_history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.history_index = Some(match self.history_index {
            None => self.history.len() - 1,
            Some(0) => self.history.len() - 1,
            Some(i) => i - 1,
        });
    }

    pub fn get_active_time(&self) -> u32 {
        // Use is_focus_mode to determine which timer to show
        if self.is_focus_mode {
            self.time_remaining_secs
        } else {
            self.rest_time_remaining_secs
        }
    }

    pub fn format_time(&self) -> String {
        let time = self.get_active_time();
        let minutes = time / 60;
        let seconds = time % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

}

impl Default for SessionInfo {
    fn default() -> Self {
        Self::new()
    }
}
