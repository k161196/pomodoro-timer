use gpui::*;
use gpui::prelude::*;
use crate::state::SessionInfo;
use crate::app::PomodoroApp;
use crate::theme::Theme;

pub struct CircularTimer {
    session_info: SessionInfo,
    label_input: String,
    is_editing_label: bool,
    view: Entity<PomodoroApp>,
    theme: Theme,
}

impl CircularTimer {
    pub fn new(
        session_info: SessionInfo,
        _sessions_until_long_break: u32,
        _total_duration_secs: u32,
        label_input: String,
        is_editing_label: bool,
        view: Entity<PomodoroApp>,
        theme: Theme,
    ) -> Self {
        Self {
            session_info,
            label_input,
            is_editing_label,
            view,
            theme,
        }
    }

    fn render_active_timer(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .w_full()
            // Focus/Rest tabs at top
            .child(self.render_tabs())
            // Compact time display
            .child(
                div()
                    .text_size(px(48.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(self.theme.foreground)
                    .child(self.session_info.format_time())
            )
            // Label in center (editable)
            .child(self.render_label_field())
            // Control buttons at bottom
            .child(self.render_control_buttons())
    }

    fn render_label_field(&self) -> impl IntoElement {
        let view = self.view.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .child(
                // Label display/input
                div()
                    .px_2()
                    .py_1()
                    .rounded(px(6.0))
                    .min_w(px(120.0))
                    .when(self.is_editing_label, |d| {
                        d.bg(rgb(0xeff6ff))
                           .border_1()
                           .border_color(rgb(0x3b82f6))
                    })
                    .when(!self.is_editing_label, |d| {
                        d.bg(self.theme.muted_background)
                    })
                    .text_size(px(13.0))
                    .text_color(self.theme.muted_foreground)
                    .text_align(TextAlign::Center)
                    .child(
                        if self.is_editing_label {
                            if self.label_input.is_empty() {
                                "|".to_string()
                            } else {
                                format!("{}|", self.label_input)
                            }
                        } else if self.session_info.current_label.is_empty() {
                            "Add label...".to_string()
                        } else {
                            self.session_info.current_label.clone()
                        }
                    )
            )
            .child(
                // Edit/Done button
                if self.is_editing_label {
                    let view_clone = view.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(24.0))
                        .rounded(px(6.0))
                        .bg(rgb(0x10b981))  // Green
                        .text_color(rgb(0xffffff))
                        .text_xs()
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0x059669)))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_done_label(cx);
                            });
                        })
                        .child("✓")
                } else {
                    let view_clone = view.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(24.0))
                        .rounded(px(6.0))
                        .bg(self.theme.secondary)
                        .text_color(self.theme.secondary_foreground)
                        .text_xs()
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0xd1d5db)))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_edit_label(cx);
                            });
                        })
                        .child("✎")
                }
            )
    }

    fn render_tabs(&self) -> impl IntoElement {
        let is_work = self.session_info.is_focus_mode;
        let view = self.view.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .p_1()
            .bg(self.theme.muted_background)
            .rounded(px(12.0))
            .child(
                // Focus tab
                {
                    let view_clone = view.clone();
                    div()
                        .px_3()
                        .py_1()
                        .rounded(px(8.0))
                        .when(is_work, |div| {
                            div.bg(self.theme.background)
                               .shadow_sm()
                        })
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if is_work { self.theme.foreground } else { self.theme.muted_foreground })
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_switch_to_focus(cx);
                            });
                        })
                        .child("Focus")
                }
            )
            .child(
                // Rest tab
                {
                    let view_clone = view.clone();
                    div()
                        .px_3()
                        .py_1()
                        .rounded(px(8.0))
                        .when(!is_work, |div| {
                            div.bg(self.theme.background)
                               .shadow_sm()
                        })
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if !is_work { self.theme.foreground } else { self.theme.muted_foreground })
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_switch_to_rest(cx);
                            });
                        })
                        .child("Rest")
                }
            )
    }


    fn render_control_buttons(&self) -> impl IntoElement {
        let is_running = self.session_info.current_state.is_running();
        let view = self.view.clone();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                // Start/Pause button
                {
                    let button_text = if is_running { "Pause" } else { "Start" };
                    let view_clone = view.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_4()
                        .py_1()
                        .rounded(px(6.0))
                        .bg(self.theme.secondary)
                        .text_color(self.theme.secondary_foreground)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_toggle(cx);
                            });
                        })
                        .child(button_text)
                }
            )
            .child(
                // Reset button
                {
                    let view_clone = view.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_4()
                        .py_1()
                        .rounded(px(6.0))
                        .bg(self.theme.secondary)
                        .text_color(self.theme.secondary_foreground)
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .hover(|style| style.opacity(0.8))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_reset(cx);
                            });
                        })
                        .child("Reset")
                }
            )
    }


    fn render_idle_state(&self) -> impl IntoElement {
        let view = self.view.clone();

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap_2()
            .w_full()
            // Focus/Rest tabs at top
            .child(self.render_tabs())
            // Compact time display
            .child(
                div()
                    .text_size(px(48.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(self.theme.foreground)
                    .child(self.session_info.format_time())
            )
            // Label in center (editable)
            .child(self.render_label_field())
            // Start button
            .child(
                {
                    let view_clone = view.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px_4()
                        .py_1()
                        .rounded(px(6.0))
                        .bg(rgb(0xe5e7eb))
                        .text_color(rgb(0x374151))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .cursor_pointer()
                        .hover(|style| style.bg(rgb(0xd1d5db)))
                        .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
                            cx.update_entity(&view_clone, |app, cx| {
                                app.handle_toggle(cx);
                            });
                        })
                        .child("Start")
                }
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
            .p_3()  // Minimal padding for compact 240x240
            .bg(self.theme.background)
            .rounded(px(16.0))  // Smaller rounded corners
            .border_2()
            .border_color(self.theme.border)
            .when(is_idle, |div| {
                div.child(self.render_idle_state())
            })
            .when(!is_idle, |div| {
                div.child(self.render_active_timer())
            })
    }
}
