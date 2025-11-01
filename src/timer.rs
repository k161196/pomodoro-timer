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
        info.is_focus_mode = true;
        // Initialize work timer if not already set
        if info.time_remaining_secs == 0 {
            info.time_remaining_secs = self.config.work_duration_secs();
        }
        info.last_updated = Utc::now();
    }

    pub async fn start_short_break(&self) {
        let mut info = self.session_info.lock().await;
        info.current_state = TimerState::ShortBreak;
        info.is_focus_mode = false;
        // Initialize rest timer if not already set
        if info.rest_time_remaining_secs == 0 {
            info.rest_time_remaining_secs = self.config.short_break_duration_secs();
        }
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

        // Reset only the current timer based on current state
        if info.current_state.is_work() || info.current_state == TimerState::Idle {
            // Reset focus timer
            info.time_remaining_secs = self.config.work_duration_secs();
        } else {
            // Reset rest timer
            info.rest_time_remaining_secs = self.config.short_break_duration_secs();
        }

        // Set to Idle but stay in same mode (focus/rest)
        info.current_state = TimerState::Idle;
        info.last_updated = Utc::now();
    }


}
