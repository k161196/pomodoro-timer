use gpui::*;

mod app;
mod config;
mod notifications;
mod persistence;
mod state;
mod theme;
mod timer;
mod ui;

use app::{PomodoroApp, QuitApp};
use config::Config;

fn main() {
    // Load configuration
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Using default configuration...");
            Config::default()
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed: {}", e);
        eprintln!("\nPlease check your config file at: ~/.config/pomodoro-timer/config.toml");
        std::process::exit(1);
    }

    Application::new().run(move |cx| {
        // Bind only quit shortcut globally
        cx.bind_keys([
            KeyBinding::new("cmd-q", QuitApp, None),
        ]);

        // Other shortcuts will be bound contextually in render to respect edit mode
        // Set window size and position - compact square
        let window_size = size(px(240.0), px(240.0));
        let centered_bounds = Bounds::centered(None, window_size, cx);

        // Open the main window as floating popup
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(centered_bounds)),
                titlebar: None,
                window_decorations: Some(WindowDecorations::Client),
                kind: WindowKind::PopUp, // Floating window
                is_movable: true,
                is_resizable: false,
                focus: true,
                show: true,
                app_id: Some("pomodoro-timer".to_string()),
                ..Default::default()
            },
            |_window, cx| cx.new(|cx| PomodoroApp::new(config.clone(), cx)),
        )
        .expect("Failed to open window");
    });
}
