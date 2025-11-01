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
    pub fn is_paused(&self) -> bool {
        matches!(
            self,
            TimerState::WorkPaused | TimerState::BreakPaused | TimerState::LongBreakPaused
        )
    }

    pub fn is_running(&self) -> bool {
        matches!(
            self,
            TimerState::Working | TimerState::ShortBreak | TimerState::LongBreak
        )
    }

    pub fn is_work(&self) -> bool {
        matches!(self, TimerState::Working | TimerState::WorkPaused)
    }

    pub fn is_break(&self) -> bool {
        matches!(
            self,
            TimerState::ShortBreak
                | TimerState::BreakPaused
                | TimerState::LongBreak
                | TimerState::LongBreakPaused
        )
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

    pub fn color_hex(&self) -> u32 {
        match self {
            TimerState::Idle => 0x6b7280,           // Gray
            TimerState::Working => 0xef4444,        // Red
            TimerState::WorkPaused => 0x9ca3af,     // Light gray
            TimerState::ShortBreak => 0x10b981,     // Green
            TimerState::BreakPaused => 0x9ca3af,    // Light gray
            TimerState::LongBreak => 0x3b82f6,      // Blue
            TimerState::LongBreakPaused => 0x9ca3af, // Light gray
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub current_state: TimerState,
    pub time_remaining_secs: u32,
    pub current_session: u32,      // Current session number (1-4)
    pub completed_sessions: u32,   // Total completed today
    pub last_updated: DateTime<Utc>,
    pub current_id: String,        // Unique ID for current timer session
    pub current_label: String,     // Label for current timer
    pub history: Vec<CompletedTimer>, // History of completed timers
    pub history_index: Option<usize>, // Current index when browsing history (None = current timer)
}

impl SessionInfo {
    pub fn new() -> Self {
        Self {
            current_state: TimerState::Idle,
            time_remaining_secs: 0,
            current_session: 1,
            completed_sessions: 0,
            last_updated: Utc::now(),
            current_id: Uuid::new_v4().to_string(),
            current_label: String::new(),
            history: Vec::new(),
            history_index: None,
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

    pub fn is_viewing_history(&self) -> bool {
        self.history_index.is_some()
    }

    pub fn get_displayed_timer(&self) -> Option<&CompletedTimer> {
        if let Some(index) = self.history_index {
            self.history.get(index)
        } else {
            None
        }
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

    pub fn navigate_history_next(&mut self) {
        if self.history.is_empty() {
            return;
        }

        self.history_index = Some(match self.history_index {
            None => 0,
            Some(i) if i >= self.history.len() - 1 => 0,
            Some(i) => i + 1,
        });
    }

    pub fn exit_history(&mut self) {
        self.history_index = None;
    }

    pub fn session_label(&self, sessions_until_long_break: u32) -> String {
        match self.current_state {
            TimerState::Working | TimerState::WorkPaused => {
                format!("Session {}/{}", self.current_session, sessions_until_long_break)
            }
            TimerState::ShortBreak | TimerState::BreakPaused => {
                "Short Break".to_string()
            }
            TimerState::LongBreak | TimerState::LongBreakPaused => {
                "Long Break".to_string()
            }
            TimerState::Idle => {
                format!("Session {}/{}", self.current_session, sessions_until_long_break)
            }
        }
    }

    pub fn format_time(&self) -> String {
        let minutes = self.time_remaining_secs / 60;
        let seconds = self.time_remaining_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn progress_percentage(&self, total_duration_secs: u32) -> f32 {
        if total_duration_secs == 0 {
            return 0.0;
        }
        let elapsed = total_duration_secs.saturating_sub(self.time_remaining_secs);
        (elapsed as f32 / total_duration_secs as f32) * 100.0
    }
}

impl Default for SessionInfo {
    fn default() -> Self {
        Self::new()
    }
}
