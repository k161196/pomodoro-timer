# Learnings and Best Practices - Pomodoro Timer

This document captures key learnings and best practices discovered while building this Pomodoro Timer application with GPUI.

## Project Overview

A Pomodoro timer application built with Rust and GPUI v0.2.0 featuring:
- Circular digital timer display
- Click-to-start/pause functionality
- State persistence across sessions
- Session tracking (Work/Short Break/Long Break)
- Desktop notifications
- Configurable durations

## GPUI Async Patterns for Timers

### Background Loops with cx.background_spawn

**Problem:** Using `tokio::time::interval` directly causes runtime errors because GPUI doesn't run on a Tokio runtime.

**Critical:** For continuous UI updates (like a countdown timer), you MUST call `cx.notify()` on every iteration when the state changes, not just when tasks complete.

**Solution:** Use `cx.background_spawn` with `std::thread::sleep` for periodic tasks:

```rust
cx.spawn(async move |this, cx| {
    loop {
        // Sleep for 1 second using background_spawn
        cx.background_spawn(async {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }).await;

        // Update timer state
        let (is_running, just_completed) = {
            let mut info = session_info.lock().await;
            if info.current_state.is_running() && info.time_remaining_secs > 0 {
                info.time_remaining_secs -= 1;
                (true, info.time_remaining_secs == 0)
            } else {
                (false, false)
            }
        };

        // CRITICAL: Call cx.notify() every second when running
        // This ensures the UI updates to show the countdown
        if is_running {
            let _ = this.update(cx, |_, cx| cx.notify());
        }

        if just_completed {
            // Handle completion (notifications, save state, etc.)
        }
    }
})
.detach();
```

**Key Points:**
- **NEVER use `tokio::time::sleep` or `tokio::time::interval`** - they require a Tokio runtime
- **USE `std::thread::sleep`** inside `cx.background_spawn()` instead
- `cx.background_spawn()` executes on GPUI's thread pool
- Call `.await` on `background_spawn()` to wait for completion
- **CRITICAL:** Call `cx.notify()` on **EVERY iteration** when state changes, not just on completion
- For countdown timers, this means calling `cx.notify()` every second to update the display
- Entities can be observed by other entities - `cx.notify()` triggers observer callbacks
- GPUI re-renders the view when `cx.notify()` is called

### Updating UI from Background Tasks

```rust
cx.spawn(async move |this, cx| {
    // Do work...

    // Update entity state
    let _ = this.update(cx, |app, cx| {
        app.session_info = new_info;
        cx.notify();  // Critical: triggers re-render
    });
})
```

**Key Points:**
- `this` is a `WeakEntity<T>` that safely references the app
- `cx` in the closure is `&mut AsyncApp`
- Pass `cx` by value (not `&mut cx`) to `update()`
- Always call `cx.notify()` to trigger UI updates

## State Management

### Persistent State with Mutex

```rust
pub struct PomodoroApp {
    session_info: Arc<Mutex<SessionInfo>>,  // Shared between background tasks
    timer: Arc<Timer>,
    config: Config,
}
```

**Why Arc<Mutex<T>>:**
- Multiple background tasks need to access/modify session state
- `Arc` provides shared ownership
- `Mutex` provides thread-safe interior mutability
- Works well with async/await

### State Persistence

Save state periodically and on changes:

```rust
// Periodic auto-save (every 5 seconds)
cx.spawn(async move |_this, cx| {
    loop {
        cx.background_spawn(async {
            std::thread::sleep(std::time::Duration::from_secs(5));
        }).await;

        let info = session_info.lock().await;
        if let Err(e) = Persistence::save(&info) {
            eprintln!("[ERROR] Failed to auto-save: {}", e);
        }
    }
})
.detach();

// Save immediately on state changes
let info = session_info.lock().await;
Persistence::save(&info)?;
```

**File locations:**
- Config: `~/.config/pomodoro-timer/config.toml`
- State: `~/.local/share/pomodoro-timer/state.json`

## UI Rendering Patterns

### Accessing Shared State in Render

**Problem:** Can't use async `.await` in `render()` method.

**Solution:** Use `blocking_lock()` for shared state access:

```rust
impl Render for PomodoroApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        // Get current session info (blocking is ok for render)
        let session_info = self.session_info.blocking_lock().clone();
        let total_duration = self.get_total_duration(&session_info.current_state);

        div()
            .child(CircularTimer::new(
                session_info,
                self.config.sessions_until_long_break,
                total_duration,
            ))
    }
}
```

**Key Points:**
- `blocking_lock()` is synchronous - OK in render methods
- Clone the data to avoid holding the lock during rendering
- Keep the lock scope minimal

### Multiple Event Handlers Reusing Entities

**Problem:** Can't move the same `Entity` into multiple closures.

**Solution:** Clone the entity for each handler:

```rust
let view_for_toggle = cx.entity().clone();
let view_for_toggle_mouse = cx.entity().clone();  // Separate clone
let view_for_skip = cx.entity().clone();

div()
    .on_action(move |_: &ToggleTimer, _window, cx| {
        cx.update_entity(&view_for_toggle, |app, cx| {
            app.handle_toggle(cx);
        });
    })
    .on_mouse_down(MouseButton::Left, move |_event, _window, cx| {
        cx.update_entity(&view_for_toggle_mouse, |app, cx| {
            app.handle_toggle(cx);
        });
    })
```

## Configuration Management

### Using TOML with Environment Variables

```rust
use std::env;

impl Config {
    pub fn config_path() -> Result<PathBuf> {
        let home = env::var("HOME")?;
        Ok(PathBuf::from(home).join(".config/pomodoro-timer/config.toml"))
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default config
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

**Key Points:**
- Auto-create config with defaults on first run
- Validate config before using (durations > 0, etc.)
- Use descriptive error messages for user-facing errors

## Hyprland Integration

### Floating Window Configuration

To make the Pomodoro timer float above all windows and pin to all workspaces:

**In main.rs:**
```rust
cx.open_window(
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(centered_bounds)),
        titlebar: None,                              // No titlebar
        window_decorations: Some(WindowDecorations::Client),
        kind: WindowKind::PopUp,                     // Floating window
        is_movable: true,
        is_resizable: false,
        focus: true,
        show: true,
        app_id: Some("pomodoro-timer".to_string()),  // Important for window rules
        ..Default::default()
    },
    |_window, cx| cx.new(|cx| PomodoroApp::new(config.clone(), cx)),
)
```

**In Hyprland config (`~/.config/hypr/hyprland.conf`):**
```hyprlang
windowrulev2 = float, class:^(pomodoro-timer)$
windowrulev2 = size 300 400, class:^(pomodoro-timer)$
windowrulev2 = center, class:^(pomodoro-timer)$
windowrulev2 = noborder, class:^(pomodoro-timer)$
windowrulev2 = noshadow, class:^(pomodoro-timer)$
windowrulev2 = pin, class:^(pomodoro-timer)$           # Pin to all workspaces
windowrulev2 = stayfocused, class:^(pomodoro-timer)$  # Keep on top
```

**Key Points:**
- Set `app_id` in WindowOptions to match Hyprland window rules
- Use `WindowKind::PopUp` for floating windows
- `pin` rule makes window visible on all workspaces
- `stayfocused` keeps window on top of others
- Remove titlebar (`titlebar: None`) for minimal UI

## Desktop Notifications

### Using notify-rust

```toml
[dependencies]
notify-rust = "4"
```

```rust
use notify_rust::Notification;

pub fn notify_work_complete() {
    let _ = Notification::new()
        .summary("Work Session Complete!")
        .body("Time for a break. Great job!")
        .timeout(5000)  // 5 seconds
        .show();
}
```

**Key Points:**
- Silently ignore notification errors (don't crash if notification daemon unavailable)
- Use descriptive summaries and bodies
- Set appropriate timeouts (5 seconds is good for timer completions)

## State Machine Design

### Clear State Transitions

```rust
#[derive(Debug, Clone, PartialEq)]
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

    pub fn pause(&self) -> Option<TimerState> {
        match self {
            TimerState::Working => Some(TimerState::WorkPaused),
            TimerState::ShortBreak => Some(TimerState::BreakPaused),
            TimerState::LongBreak => Some(TimerState::LongBreakPaused),
            _ => None,
        }
    }
}
```

**Benefits:**
- Type-safe state transitions
- Easy to test and reason about
- Self-documenting code
- Compile-time guarantees

## How GPUI Rendering Works

### The Render Cycle

GPUI uses a **hybrid immediate and retained mode** rendering system:

1. **At the start of each frame**, GPUI calls `render()` on the root view
2. Views return a tree of `Element`s that get laid out and styled
3. GPUI only re-renders when explicitly told via `cx.notify()`

### Entity Observation System

From GPUI documentation:
> "Entities can be observed by other entities and windows, allowing a closure to be called when `notify` is called on the entity's `Context`."

This means:
- Calling `cx.notify()` triggers **all observers** of that entity
- Windows observing the entity will **re-render**
- Other entities observing will have their **callbacks invoked**

### Why UI Doesn't Auto-Update

**GPUI is NOT reactive by default** - it won't automatically detect state changes. You must explicitly call `cx.notify()`:

```rust
// State changes but UI doesn't update
let mut info = self.session_info.lock().await;
info.time_remaining_secs -= 1;  // UI won't update!

// Correct - notify triggers re-render
let mut info = self.session_info.lock().await;
info.time_remaining_secs -= 1;
cx.notify();  // Now UI updates
```

### No Built-in Timer API

GPUI does **not** provide `spawn_timer()` or similar:
- Use `cx.spawn()` for async tasks
- Use `cx.background_spawn()` for blocking operations
- Build your own timer loops with `std::thread::sleep`

From GPUI docs: "The best way to learn about these APIs is to read the Zed source code."

## Common Pitfalls

### ❌ Don't: Use Tokio Runtime Functions

```rust
// WRONG - causes "no reactor running" error
let mut interval = tokio::time::interval(Duration::from_secs(1));
interval.tick().await;
```

### ✅ Do: Use GPUI Background Spawn

```rust
// CORRECT - works with GPUI's executor
cx.background_spawn(async {
    std::thread::sleep(std::time::Duration::from_secs(1));
}).await;
```

### ❌ Don't: Forget cx.notify()

```rust
// WRONG - UI won't update
this.update(cx, |app, cx| {
    app.state = new_state;
    // Missing cx.notify()!
});
```

### ✅ Do: Always Call cx.notify()

```rust
// CORRECT - UI updates immediately
this.update(cx, |app, cx| {
    app.state = new_state;
    cx.notify();  // Triggers re-render
});
```

### ❌ Don't: Only Call cx.notify() on Completion

```rust
// WRONG - UI only updates when timer completes
loop {
    sleep(1_second).await;
    update_timer_state();

    if timer_completed {
        cx.notify();  // UI frozen for entire duration!
    }
}
```

### ✅ Do: Call cx.notify() on Every State Change

```rust
// CORRECT - UI updates every second
loop {
    cx.background_spawn(async {
        std::thread::sleep(Duration::from_secs(1));
    }).await;

    let is_running = update_timer_state();

    if is_running {
        let _ = this.update(cx, |_, cx| cx.notify());  // Updates every tick
    }
}
```

**Why this matters:**
- GPUI doesn't auto-detect state changes
- Without `cx.notify()`, the UI shows stale data
- For timers/countdowns, you need to notify on **every tick**
- This is different from frameworks like React that auto-detect changes

## Learnings from Zed Source Code Research

### What We Discovered

When researching how Zed implements timers and periodic updates, we found:

1. **No `spawn_timer()` method exists** in GPUI's App or Context structs
2. **Executors are the foundation**: Use `App::spawn()`, `background_executor()`, and `foreground_executor()`
3. **Entity observation is key**: The `cx.notify()` mechanism is what triggers UI updates
4. **Build your own timers**: Combine `cx.spawn()` with async loops and `std::thread::sleep`

### Official GPUI Documentation Quotes

From `zed/crates/gpui/src/app.rs`:
> "Spawns the future returned by the given function on the main thread. The closure will be invoked with AsyncApp, which allows the application state to be accessed across await points."

From `zed/crates/gpui/docs/contexts.md`:
> "Entities can be observed by other entities and windows, allowing a closure to be called when notify is called on the entity's Context."

### The GPUI Philosophy

GPUI is **explicitly non-reactive**:
- You must manually trigger updates with `cx.notify()`
- This gives you precise control over when rendering happens
- Trade-off: More verbose, but better performance control
- Different from React/SwiftUI where changes auto-propagate

### Key Takeaway

The Zed documentation literally says:
> "The best way to learn about these APIs is to read the Zed source code."

This confirms that GPUI is intended to be learned by example, not comprehensive API docs. Patterns like our timer implementation follow Zed's approach.

## Resources

### Documentation
- **GPUI v0.2.0 Documentation**: https://docs.rs/gpui/0.2.0/gpui/
- **Zed Editor Source**: https://github.com/zed-industries/zed
- **GPUI Contexts Guide**: https://github.com/zed-industries/zed/blob/main/crates/gpui/docs/contexts.md
- **Rust Async Book**: https://rust-lang.github.io/async-book/

### Key Files for Reference
- GPUI async patterns: Search Zed repo for `cx.background_spawn` usage
- GPUI contexts: `zed/crates/gpui/docs/contexts.md`
- GPUI app source: `zed/crates/gpui/src/app.rs`
- State management examples: Zed's UI components

## Testing

To test the application:

```bash
# Build and run
cargo run

# Build release version
cargo build --release

# Run release version
./target/release/pomodoro-timer
```

The app window will appear and you can:
- Click or press Space to start/pause
- Press S to skip to next session
- Press ESC to reset
- Press Cmd+Q to quit

State is saved to `~/.local/share/pomodoro-timer/state.json` automatically.
