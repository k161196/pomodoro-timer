use gpui::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;

use crate::config::Config;
use crate::notifications;
use crate::persistence::Persistence;
use crate::state::{SessionInfo, TimerState};
use crate::timer::Timer;
use crate::ui::CircularTimer;

actions!(pomodoro, [ToggleTimer, ResetTimer, SkipSession, QuitApp, NewTimer, NavigateHistoryPrev, NavigateHistoryNext]);

pub struct PomodoroApp {
    session_info: Arc<Mutex<SessionInfo>>,
    timer: Arc<Timer>,
    config: Config,
    focus_handle: FocusHandle,
    label_input: String,  // Current text in label input field
    is_editing_label: bool,  // True when actively editing label
}

impl PomodoroApp {
    pub fn new(config: Config, cx: &mut Context<'_, Self>) -> Self {
        // Load persisted state
        let session_info = match Persistence::load() {
            Ok(info) => {
                notifications::log_info("Loaded persisted timer state");
                Arc::new(Mutex::new(info))
            }
            Err(e) => {
                notifications::log_error(&format!("Failed to load state: {}", e));
                Arc::new(Mutex::new(SessionInfo::new()))
            }
        };

        let timer = Arc::new(Timer::new(session_info.clone(), config.clone()));

        // Spawn background tick loop using background_spawn
        let session_info_for_tick = session_info.clone();
        let config_for_tick = config.clone();
        cx.spawn(async move |this, cx| {
            loop {
                // Sleep for 1 second using background_spawn
                cx.background_spawn(async {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }).await;

                let (is_running, just_completed) = {
                    let mut info = session_info_for_tick.lock().await;
                    let is_running = info.current_state.is_running();

                    let just_completed = if is_running && info.time_remaining_secs > 0 {
                        info.time_remaining_secs -= 1;
                        info.last_updated = Utc::now();
                        info.time_remaining_secs == 0
                    } else {
                        false
                    };

                    (is_running, just_completed)
                };

                // Trigger UI update every second when running
                if is_running {
                    let _ = this.update(cx, |_, cx| cx.notify());
                }

                if just_completed {
                    notifications::log_info("Timer completed!");

                    // Send notification
                    let info = session_info_for_tick.lock().await;
                    if config_for_tick.enable_notifications {
                        match info.current_state {
                            TimerState::Working => notifications::notify_work_complete(),
                            TimerState::ShortBreak => notifications::notify_break_complete(),
                            TimerState::LongBreak => notifications::notify_long_break_complete(),
                            _ => {}
                        }
                    }

                    // Save state
                    if let Err(e) = Persistence::save(&info) {
                        notifications::log_error(&format!("Failed to save state: {}", e));
                    }
                }
            }
        })
        .detach();

        // Periodically save state
        let session_info_clone = session_info.clone();
        cx.spawn(async move |_this, cx| {
            loop {
                // Sleep for 5 seconds using background_spawn
                cx.background_spawn(async {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }).await;

                let info = session_info_clone.lock().await;
                if let Err(e) = Persistence::save(&info) {
                    notifications::log_error(&format!("Failed to auto-save state: {}", e));
                }
            }
        })
        .detach();

        Self {
            session_info,
            timer,
            config,
            focus_handle: cx.focus_handle(),
            label_input: String::new(),
            is_editing_label: false,
        }
    }

    pub fn handle_new_timer(&mut self, cx: &mut Context<'_, Self>) {
        let session_info = self.session_info.clone();
        let label = self.label_input.clone();

        cx.spawn(async move |this, cx| {
            use uuid::Uuid;

            // Reset to idle and create new timer with new ID
            {
                let mut info = session_info.lock().await;
                info.current_state = TimerState::Idle;
                info.time_remaining_secs = 0;
                info.current_id = Uuid::new_v4().to_string();  // Generate new UUID
                info.current_label = label;
                info.exit_history();
                info.last_updated = Utc::now();
            }

            // Save state
            let info = session_info.lock().await;
            if let Err(e) = Persistence::save(&info) {
                notifications::log_error(&format!("Failed to save state: {}", e));
            }

            // Clear label input and trigger UI update
            let _ = this.update(cx, |app, cx| {
                app.label_input.clear();
                cx.notify();
            });
        })
        .detach();
    }

    pub fn handle_toggle(&mut self, cx: &mut Context<'_, Self>) {
        let timer = self.timer.clone();
        let session_info = self.session_info.clone();

        cx.spawn(async move |this, cx| {
            let current_state = {
                let info = session_info.lock().await;
                info.current_state.clone()
            };

            match current_state {
                TimerState::Idle => {
                    // Start first work session
                    timer.start_work().await;
                    notifications::log_info("Started work session");
                }
                TimerState::Working | TimerState::ShortBreak | TimerState::LongBreak => {
                    // Pause
                    timer.pause().await;
                    notifications::log_info("Paused timer");
                }
                TimerState::WorkPaused | TimerState::BreakPaused | TimerState::LongBreakPaused => {
                    // Resume
                    timer.resume().await;
                    notifications::log_info("Resumed timer");
                }
            }

            // Save state
            let info = session_info.lock().await;
            if let Err(e) = Persistence::save(&info) {
                notifications::log_error(&format!("Failed to save state: {}", e));
            }

            // Trigger UI update
            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn handle_skip(&mut self, cx: &mut Context<'_, Self>) {
        let session_info = self.session_info.clone();

        cx.spawn(async move |this, cx| {
            // Stop timer and add to history, then navigate to previous
            {
                let mut info = session_info.lock().await;

                // If running, stop and add to history
                if info.current_state.is_running() {
                    let session_type = info.current_state.display_name().to_string();
                    let elapsed = info.time_remaining_secs;
                    let id = info.current_id.clone();
                    let label = info.current_label.clone();

                    info.add_to_history(id, label, elapsed, session_type);
                    info.current_state = TimerState::Idle;
                    info.time_remaining_secs = 0;
                    notifications::log_info("Timer stopped and saved to history");
                }

                // Navigate to previous in history
                info.navigate_history_prev();
            }

            // Save state
            let info = session_info.lock().await;
            if let Err(e) = Persistence::save(&info) {
                notifications::log_error(&format!("Failed to save state: {}", e));
            }

            // Trigger UI update
            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn handle_navigate_history_prev(&mut self, cx: &mut Context<'_, Self>) {
        let session_info = self.session_info.clone();

        cx.spawn(async move |this, cx| {
            {
                let mut info = session_info.lock().await;
                info.navigate_history_prev();
            }

            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn handle_navigate_history_next(&mut self, cx: &mut Context<'_, Self>) {
        let session_info = self.session_info.clone();

        cx.spawn(async move |this, cx| {
            {
                let mut info = session_info.lock().await;
                info.navigate_history_next();
            }

            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn handle_reset(&mut self, cx: &mut Context<'_, Self>) {
        let timer = self.timer.clone();
        let session_info = self.session_info.clone();

        cx.spawn(async move |this, cx| {
            timer.reset().await;
            notifications::log_info("Reset timer");

            // Save state
            let info = session_info.lock().await;
            if let Err(e) = Persistence::save(&info) {
                notifications::log_error(&format!("Failed to save state: {}", e));
            }

            // Trigger UI update
            let _ = this.update(cx, |_, cx| cx.notify());
        })
        .detach();
    }

    pub fn handle_edit_label(&mut self, cx: &mut Context<'_, Self>) {
        // Load current label into edit buffer and enter edit mode
        let current_label = self.session_info.blocking_lock().current_label.clone();
        self.label_input = current_label;
        self.is_editing_label = true;
        cx.notify();
    }

    pub fn handle_done_label(&mut self, cx: &mut Context<'_, Self>) {
        // Save label and exit edit mode
        let label = self.label_input.clone();
        let session_info = self.session_info.clone();

        cx.spawn(async move |_this, cx| {
            {
                let mut info = session_info.lock().await;
                info.current_label = label;
            }
            let _ = cx.update(|_cx| {});
        }).detach();

        self.label_input.clear();
        self.is_editing_label = false;
        cx.notify();
    }

    fn get_total_duration(&self, state: &TimerState) -> u32 {
        match state {
            TimerState::Working | TimerState::WorkPaused => self.config.work_duration_secs(),
            TimerState::ShortBreak | TimerState::BreakPaused => {
                self.config.short_break_duration_secs()
            }
            TimerState::LongBreak | TimerState::LongBreakPaused => {
                self.config.long_break_duration_secs()
            }
            TimerState::Idle => 0,
        }
    }
}

impl Render for PomodoroApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let view_for_keyboard = cx.entity().clone();
        let view_for_ui = cx.entity().clone();
        let focus_handle = self.focus_handle.clone();

        // Request focus
        self.focus_handle.focus(window);

        // Get current session info (blocking is ok for render)
        let session_info = self.session_info.blocking_lock().clone();
        let total_duration = self.get_total_duration(&session_info.current_state);
        let is_editing = self.is_editing_label;

        div()
            .w_full()
            .h_full()
            .track_focus(&focus_handle)
            .on_key_down(move |event, _window, cx| {
                let keystroke = &event.keystroke;
                let key = keystroke.key.as_str();

                // Check edit state once
                let is_editing = cx.update_entity(&view_for_keyboard, |app, _cx| app.is_editing_label);

                if is_editing {
                    // EDIT MODE: Only handle text input, block all shortcuts
                    cx.update_entity(&view_for_keyboard, |app, cx| {
                        if key == "backspace" {
                            app.label_input.pop();
                            cx.notify();
                        } else if key.len() == 1 && app.label_input.len() < 30 {
                            app.label_input.push_str(key);
                            cx.notify();
                        }
                        // All other keys (including s, space, escape) are ignored
                    });
                } else {
                    // NOT EDITING: Handle shortcuts
                    match key {
                        "space" => {
                            cx.update_entity(&view_for_keyboard, |app, cx| app.handle_toggle(cx));
                        }
                        "s" => {
                            cx.update_entity(&view_for_keyboard, |app, cx| app.handle_skip(cx));
                        }
                        "escape" => {
                            cx.update_entity(&view_for_keyboard, |app, cx| app.handle_reset(cx));
                        }
                        "n" => {
                            cx.update_entity(&view_for_keyboard, |app, cx| app.handle_new_timer(cx));
                        }
                        _ => {}
                    }
                }
            })
            .on_action(|_: &QuitApp, _window, cx| {
                cx.quit();
            })
            .child(CircularTimer::new(
                session_info,
                self.config.sessions_until_long_break,
                total_duration,
                self.label_input.clone(),
                is_editing,
                view_for_ui,
            ))
    }
}
