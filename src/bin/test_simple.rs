use gpui::*;

struct SimpleTest {}

impl SimpleTest {
    pub fn new(_cx: &mut Context<'_, Self>) -> Self {
        Self {}
    }
}

impl Render for SimpleTest {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .p_0()  // Remove all padding
            .m_0()  // Remove all margin
            .bg(rgb(0x222222))
            .child(
                // Left column
                div()
                    .flex()
                    .flex_col()
                    .w(relative(0.33))
                    .h_full()
                    .child(
                        div()
                            .h(relative(0.33))
                            .bg(rgb(0xFF0000))
                    )
            )
            .child(
                // Middle column
                div()
                    .flex()
                    .flex_col()
                    .w(relative(0.33))
                    .h_full()
                    .child(
                        div()
                            .h(relative(0.33))
                            .bg(rgb(0x00FF00))
                    )
            )
            .child(
                // Right column
                div()
                    .flex()
                    .flex_col()
                    .w(relative(0.34))
                    .h_full()
                    .child(
                        div()
                            .h(relative(0.33))
                            .bg(rgb(0x0000FF))
                    )
            )
    }
}

fn main() {
    Application::new().run(move |cx| {
        let screen_bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or_else(|| {
            Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(1920.0), px(1080.0)),
            }
        });

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(screen_bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                kind: WindowKind::PopUp,
                is_movable: false,
                is_resizable: false,
                focus: false,
                show: true,
                app_id: Some("pomodoro-celebration".to_string()),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| SimpleTest::new(cx)),
        )
        .expect("Failed to open window");
    });
}
