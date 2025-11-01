use gpui::*;

// Copy the celebration module code here for standalone testing
mod celebration {
    use gpui::*;
    use std::time::Duration;

    /// Celebration animation component shown when timer completes
    pub struct CelebrationWindow {}

    impl CelebrationWindow {
        pub fn new(cx: &mut Context<'_, Self>) -> Self {
            // Auto-quit after 10 seconds (for test only)
            cx.spawn(async move |_this, cx| {
                cx.background_spawn(async {
                    std::thread::sleep(Duration::from_secs(10));
                })
                .await;

                let _ = cx.update(|cx| cx.quit());
            })
            .detach();

            Self {}
        }

        /// Create and show a celebration window
        pub fn show(cx: &mut App) -> Result<WindowHandle<CelebrationWindow>> {
            // Get screen dimensions for full-screen overlay
            let screen_bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or_else(|| {
                Bounds {
                    origin: point(px(0.0), px(0.0)),
                    size: size(px(1920.0), px(1080.0)),
                }
            });

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Fullscreen(screen_bounds)),
                    titlebar: None,
                    window_decorations: None,
                    kind: WindowKind::Normal,
                    is_movable: false,
                    is_resizable: false,
                    focus: false,
                    show: true,
                    app_id: Some("pomodoro-celebration".to_string()),
                    ..Default::default()
                },
                |_window, cx| cx.new(|cx| Self::new(cx)),
            )
        }
    }

    /// Pseudo-random number generator for particle positions
    fn pseudo_random(seed: usize) -> f32 {
        // Simple pseudo-random using golden ratio
        let phi = 1.618033988749895;
        (seed as f32 * phi) % 1.0
    }

    impl Render for CelebrationWindow {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
            // Create confetti pieces
            let confetti: Vec<_> = (0..100)
                .map(|i| {
                    // Random colors
                    let color = match i % 10 {
                        0 => rgb(0xFF6B6B), // Red
                        1 => rgb(0x4ECDC4), // Teal
                        2 => rgb(0xFFE66D), // Yellow
                        3 => rgb(0x95E1D3), // Mint
                        4 => rgb(0xF38181), // Pink
                        5 => rgb(0xAA96DA), // Purple
                        6 => rgb(0xFF8E53), // Orange
                        7 => rgb(0x6BCF7F), // Green
                        8 => rgb(0x5DADE2), // Blue
                        _ => rgb(0xF78FB3),  // Rose
                    };

                    // Random horizontal position (0-100%)
                    let x_pos = pseudo_random(i * 17) * 100.0;

                    // Random size (width and height for rectangular confetti)
                    let width = 8.0 + pseudo_random(i * 31) * 8.0; // 8-16px
                    let height = 6.0 + pseudo_random(i * 37) * 6.0; // 6-12px

                    // Random fall speed
                    let duration_ms = 2000 + (pseudo_random(i * 41) * 3000.0) as u64; // 2-5 seconds

                    // Random horizontal sway
                    let sway_amount = (pseudo_random(i * 53) - 0.5) * 15.0; // -7.5% to +7.5%

                    div()
                        .absolute()
                        .left(relative(x_pos / 100.0))
                        .w(px(width))
                        .h(px(height))
                        .bg(color)
                        .rounded(px(2.0))
                        .with_animation(
                            ("confetti", i),
                            Animation::new(Duration::from_millis(duration_ms)).repeat(),
                            move |this, delta| {
                                // Fall from top (-10%) to bottom (110%)
                                let y_pos = -10.0 + (delta * 120.0);

                                // Horizontal sway (sine wave)
                                let sway = (delta * std::f32::consts::PI * 4.0).sin() * sway_amount;
                                let x_offset = x_pos + sway;

                                // Rotation for tumbling effect
                                let rotation = delta * 360.0 * 3.0; // 3 full rotations

                                // Fade in at start, fade out at end
                                let opacity = if delta < 0.05 {
                                    delta / 0.05
                                } else if delta > 0.85 {
                                    1.0 - ((delta - 0.85) / 0.15)
                                } else {
                                    1.0
                                };

                                this.left(relative(x_offset / 100.0))
                                    .top(relative(y_pos / 100.0))
                                    .opacity(opacity.max(0.0))
                            },
                        )
                })
                .collect();

            div()
                .w_full()
                .h_full()
                .relative()
                .p_0()
                .m_0()
                .bg(rgba(0x00000001)) // Nearly transparent (opacity: 1/255)
                .children(confetti)
        }
    }
}

use celebration::CelebrationWindow;

fn main() {
    Application::new().run(move |cx| {
        // Open the celebration window immediately
        CelebrationWindow::show(cx).expect("Failed to show celebration window");
    });
}
