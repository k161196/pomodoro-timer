use gpui::*;
use gpui::prelude::*;
use crate::state::SessionInfo;
use crate::app::PomodoroApp;

pub struct CircularTimer {
    session_info: SessionInfo,
    sessions_until_long_break: u32,
    total_duration_secs: u32,
    label_input: String,
    is_editing_label: bool,
    view: Entity<PomodoroApp>,
}

impl CircularTimer {
    pub fn new(
        session_info: SessionInfo,
        sessions_until_long_break: u32,
        total_duration_secs: u32,
        label_input: String,
        is_editing_label: bool,
        view: Entity<PomodoroApp>,
    ) -> Self {
        Self {
            session_info,
            sessions_until_long_break,
            total_duration_secs,
            label_input,
            is_editing_label,
            view,
        }
    }

    fn render_progress_ring(&self) -> impl IntoElement {
        let progress = self.session_info.progress_percentage(self.total_duration_secs);
        let color = self.session_info.current_state.color_hex();

        // Outer container for the circular progress
        div()
            .relative()
            .size(px(200.0))
            .flex()
            .items_center()
            .justify_center()
            // Background circle (gray track)
            .child(
                div()
                    .absolute()
                    .size(px(200.0))
                    .rounded_full()
                    .border_8()
                    .border_color(rgb(0x374151))
            )
            // Progress circle (colored arc)
            .child(self.render_progress_arc(progress, color))
            // Center content
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .child(
                        // Time display
                        div()
                            .text_size(px(36.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xffffff))
                            .child(self.session_info.format_time())
                    )
                    .child(
                        // Label and edit/done button row
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                // Editable label display
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded(px(4.0))
                                    .border_2()
                                    .when(self.is_editing_label, |div| {
                                        div.bg(rgb(0x1f2937))  // Darker background when editing
                                           .border_color(rgb(0x3b82f6))  // Blue border when editing
                                    })
                                    .when(!self.is_editing_label, |div| {
                                        div.bg(rgb(0x374151))  // Normal background
                                           .border_color(rgb(0x4b5563))  // Gray border
                                    })
                                    .text_sm()
                                    .text_color(rgb(0xe5e7eb))
                                    .text_align(TextAlign::Center)
                                    .min_w(px(100.0))
                                    .child(
                                        if self.is_editing_label {
                                            // Editing mode - show input with cursor
                                            if self.label_input.is_empty() {
                                                "|".to_string()
                                            } else {
                                                format!("{}|", self.label_input)
                                            }
                                        } else if self.session_info.current_label.is_empty() {
                                            // Not editing, no label - show placeholder
                                            "...".to_string()
                                        } else {
                                            // Not editing, has label - show label
                                            self.session_info.current_label.clone()
                                        }
                                    )
                            )
                            .child(
                                // Edit/Done button
                                self.render_edit_done_button()
                            )
                    )
                    .child(
                        // Play/Pause button below label
                        if let Some(button) = self.render_play_pause_button() {
                            button.into_any_element()
                        } else {
                            div().into_any_element()
                        }
                    )
            )
    }

    fn render_progress_arc(&self, progress: f32, color: u32) -> impl IntoElement {
        // For now, we'll use a simple border overlay with opacity
        // A proper SVG arc would be better, but GPUI's canvas API is limited
        let opacity = (progress / 100.0).min(1.0);

        div()
            .absolute()
            .size(px(200.0))
            .rounded_full()
            .border_8()
            .border_color(rgb(color))
            .opacity(opacity)
    }

    fn render_edit_done_button(&self) -> impl IntoElement {
        let view = self.view.clone();

        if self.is_editing_label {
            // Show Done button
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_2()
                .py_1()
                .rounded(px(4.0))
                .bg(rgb(0x10b981))  // Green
                .text_color(rgb(0xffffff))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .hover(|style| style.opacity(0.8))
                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                    cx.update_entity(&view, |app, cx| {
                        app.handle_done_label(cx);
                    });
                })
                .child("✓")
        } else {
            // Show Edit button
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_2()
                .py_1()
                .rounded(px(4.0))
                .bg(rgb(0x6b7280))  // Gray
                .text_color(rgb(0xffffff))
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .hover(|style| style.opacity(0.8))
                .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                    cx.update_entity(&view, |app, cx| {
                        app.handle_edit_label(cx);
                    });
                })
                .child("✎")
        }
    }

    fn render_play_pause_button(&self) -> Option<impl IntoElement> {
        let is_idle = matches!(self.session_info.current_state, crate::state::TimerState::Idle);
        let is_running = self.session_info.current_state.is_running();

        if !is_idle {
            let button_text = if is_running { "||" } else { "▶" };
            let button_color = if is_running { 0xf59e0b } else { 0x10b981 };
            let view = self.view.clone();

            Some(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(36.0))
                    .rounded_full()
                    .bg(rgb(button_color))
                    .text_color(rgb(0xffffff))
                    .text_size(px(14.0))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|style| style.opacity(0.8))
                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                        cx.update_entity(&view, |app, cx| {
                            app.handle_toggle(cx);
                        });
                    })
                    .child(button_text.to_string())
            )
        } else {
            None
        }
    }

    fn render_button(&self, label: &str, color: u32) -> impl IntoElement {
        div()
            .px_4()
            .py_2()
            .rounded(px(8.0))
            .bg(rgb(color))
            .text_sm()
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(0xffffff))
            .cursor_pointer()
            .hover(|style| style.opacity(0.8))
            .child(label.to_string())
    }

    fn render_idle_state(&self) -> impl IntoElement {
        let view = self.view.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .child(
                div()
                    .size(px(160.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .border_8()
                    .border_color(rgb(0x374151))
                    .cursor_pointer()
                    .hover(|style| style.border_color(rgb(0x4b5563)))
                    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                        cx.update_entity(&view, |app, cx| {
                            app.handle_toggle(cx);
                        });
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(24.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x9ca3af))
                                    .child("00:00")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6b7280))
                                    .child("Tap to start")
                            )
                    )
            )
    }
}

impl IntoElement for CircularTimer {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        let is_idle = matches!(self.session_info.current_state, crate::state::TimerState::Idle);

        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_6()
            .bg(rgb(0x111827))
            .when(is_idle, |div| {
                div.child(self.render_idle_state())
            })
            .when(!is_idle, |div| {
                div.child(self.render_progress_ring())
            })
    }
}
