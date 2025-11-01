use crate::config::Config;
use crate::state::{SessionInfo, TimerState};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Timer {
    session_info: Arc<Mutex<SessionInfo>>,
    config: Config,
}

impl Timer {
    pub fn new(session_info: Arc<Mutex<SessionInfo>>, config: Config) -> Self {
        Self {
            session_info,
            config,
        }
    }

    pub async fn start_work(&self) {
        let mut info = self.session_info.lock().await;
        info.current_state = TimerState::Working;
        info.time_remaining_secs = self.config.work_duration_secs();
        info.last_updated = Utc::now();
    }

    pub async fn start_short_break(&self) {
        let mut info = self.session_info.lock().await;
        info.current_state = TimerState::ShortBreak;
        info.time_remaining_secs = self.config.short_break_duration_secs();
        info.last_updated = Utc::now();
    }

    pub async fn start_long_break(&self) {
        let mut info = self.session_info.lock().await;
        info.current_state = TimerState::LongBreak;
        info.time_remaining_secs = self.config.long_break_duration_secs();
        info.last_updated = Utc::now();
    }

    pub async fn pause(&self) {
        let mut info = self.session_info.lock().await;
        if let Some(paused_state) = info.current_state.pause() {
            info.current_state = paused_state;
            info.last_updated = Utc::now();
        }
    }

    pub async fn resume(&self) {
        let mut info = self.session_info.lock().await;
        if let Some(resumed_state) = info.current_state.resume() {
            info.current_state = resumed_state;
            info.last_updated = Utc::now();
        }
    }

    pub async fn reset(&self) {
        let mut info = self.session_info.lock().await;
        info.current_state = TimerState::Idle;
        info.time_remaining_secs = 0;
        info.last_updated = Utc::now();
    }

    pub async fn skip_to_next(&self) {
        let mut info = self.session_info.lock().await;

        match info.current_state {
            TimerState::Working | TimerState::WorkPaused => {
                // Work session completed, move to break
                info.completed_sessions += 1;

                // Check if we should do long break
                if info.current_session >= self.config.sessions_until_long_break {
                    info.current_state = TimerState::LongBreak;
                    info.time_remaining_secs = self.config.long_break_duration_secs();
                    info.current_session = 1; // Reset to session 1
                } else {
                    info.current_state = TimerState::ShortBreak;
                    info.time_remaining_secs = self.config.short_break_duration_secs();
                    info.current_session += 1; // Increment for next work session
                }
            }
            TimerState::ShortBreak | TimerState::BreakPaused |
            TimerState::LongBreak | TimerState::LongBreakPaused => {
                // Break completed, move to work
                info.current_state = TimerState::Working;
                info.time_remaining_secs = self.config.work_duration_secs();
            }
            TimerState::Idle => {
                // From idle, start first work session
                info.current_state = TimerState::Working;
                info.time_remaining_secs = self.config.work_duration_secs();
                info.current_session = 1;
            }
        }

        info.last_updated = Utc::now();
    }

}
