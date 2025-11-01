# Pomodoro Timer

A beautiful, minimalist Pomodoro timer application built with Rust and GPUI. Features a circular digital timer display with persistent state across sessions.

## Features

- ⏱️ **Classic Pomodoro Technique** - 25-min work, 5-min breaks, 15-min long breaks
- 🔄 **State Persistence** - Your timer state is saved, resume anytime
- 🎯 **Circular Timer Display** - Clean, digital countdown with progress ring
- 🔔 **Desktop Notifications** - Get notified when sessions complete
- ⚙️ **Configurable** - Customize session durations
- ⌨️ **Keyboard Shortcuts** - Fast navigation with hotkeys
- 📊 **Session Tracking** - See which session you're on (1/4, 2/4, etc.)

## Prerequisites

### Linux
- Rust (latest stable)
- GPUI v0.2.0 dependencies
- Desktop notification support

### macOS
- Rust (latest stable)
- Xcode Command Line Tools: `xcode-select --install`
- macOS 10.15 (Catalina) or later

**Note:** macOS requires Metal rendering backend, which is included with Xcode Command Line Tools.

## Installation

### Linux

#### 1. Clone and Build

```bash
cd ~/projects/pomodoro-timer
cargo build --release
```

### macOS

#### 1. Install Prerequisites

```bash
# Install Xcode Command Line Tools (if not already installed)
xcode-select --install

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Clone and Build

```bash
cd ~/projects/pomodoro-timer
cargo build --release
```

**macOS Build Notes:**
- First build may take longer as it compiles Metal shaders
- The app uses native macOS notifications
- Window management respects macOS system preferences

#### 2. Install Desktop File (Linux Only - Optional)

Create a desktop entry for easy launching:

**Create the file:** `~/.local/share/applications/pomodoro-timer.desktop`

```desktop
[Desktop Entry]
Name=Pomodoro Timer
Comment=A minimalist Pomodoro timer built with Rust and GPUI
Exec=/home/YOUR_USERNAME/projects/pomodoro-timer/target/release/pomodoro-timer
Icon=clock
Type=Application
Categories=Utility;
Terminal=false
StartupWMClass=pomodoro-timer
```

**Note:** Replace `/home/YOUR_USERNAME/` with your actual home path or use `~/projects/pomodoro-timer/target/release/pomodoro-timer`.

After creating the file, update the desktop database:
```bash
update-desktop-database ~/.local/share/applications/
```

#### 3. Configure Hyprland (Linux Only - Optional but Recommended)

Add the following to your Hyprland config (`~/.config/hypr/hyprland.conf`) for the best experience:

```hyprlang
# Pomodoro Timer - configure as floating window that stays on all workspaces
windowrulev2 = float, class:^(pomodoro-timer)$
windowrulev2 = size 240 240, class:^(pomodoro-timer)$
windowrulev2 = center, class:^(pomodoro-timer)$
windowrulev2 = noborder, class:^(pomodoro-timer)$
windowrulev2 = noshadow, class:^(pomodoro-timer)$
windowrulev2 = pin, class:^(pomodoro-timer)$           # Pin to all workspaces
windowrulev2 = noblur, class:^(pomodoro-timer)$

# Optional: Add a keybind to launch
bind = SUPER, P, exec, ~/projects/pomodoro-timer/target/release/pomodoro-timer
```

**What these rules do:**
- `float` - Makes the window float above other windows
- `size 240 240` - Sets the window size to 240x240 pixels (compact square)
- `center` - Centers the window on screen
- `noborder` / `noshadow` - Removes decorations for a clean look
- `pin` - **Pins the window to all workspaces** (stays visible when switching workspaces)
- `noblur` - Disables background blur for better performance

## Running the App

### Linux

```bash
./target/release/pomodoro-timer
```

Or use the Hyprland keybind (Super+P) if configured above.

### macOS

```bash
./target/release/pomodoro-timer
```

**macOS Notes:**
- The app will request notification permissions on first launch
- Window can be moved and resized like any native macOS app
- Use Cmd+Q to quit
- Consider adding to Login Items for auto-start (System Preferences → Users & Groups → Login Items)

## Usage

### Starting a Session

1. **Click the timer circle** or press **Space** to start your first work session
2. The timer will count down from 25:00
3. A progress ring shows your progress visually

### Controls

**Keyboard Shortcuts:**
- **Space** - Start/Pause the current timer
- **S** - Skip to next session
- **ESC** - Reset current session to idle
- **Cmd+Q** - Quit the application

**Mouse:**
- **Click timer** - Start/Pause
- **Click Pause button** - Pause current session
- **Click Skip button** - Move to next session (break or work)
- **Click Resume button** - Resume paused session

### Session Flow

1. **Work Session** (25 min) → **Short Break** (5 min)
2. **Work Session** (25 min) → **Short Break** (5 min)
3. **Work Session** (25 min) → **Short Break** (5 min)
4. **Work Session** (25 min) → **Long Break** (15 min)
5. Cycle repeats from session 1

### State Persistence

Your timer state is automatically saved to `~/.local/share/pomodoro-timer/state.json`:
- Current timer state (working, break, paused)
- Time remaining
- Current session number
- Total completed sessions

You can close and reopen the app without losing your progress!

## Configuration

Configuration file: `~/.config/pomodoro-timer/config.toml`

The config file is automatically created with defaults on first run.

### Default Configuration

```toml
# Session durations (in minutes)
work_duration = 25
short_break_duration = 5
long_break_duration = 15

# Number of work sessions before long break
sessions_until_long_break = 4

# Notifications
enable_notifications = true

# Auto-start (manual control by default)
auto_start_breaks = false
auto_start_work = false
```

### Customization Examples

**Shorter sessions for testing:**
```toml
work_duration = 1
short_break_duration = 1
long_break_duration = 2
```

**Extended focus mode:**
```toml
work_duration = 50
short_break_duration = 10
long_break_duration = 30
sessions_until_long_break = 3
```

**Disable notifications:**
```toml
enable_notifications = false
```

### Resetting Configuration

To reset to defaults:
```bash
rm ~/.config/pomodoro-timer/config.toml
```

The app will recreate it on next launch.

## UI States

| State | Description | Display |
|-------|-------------|---------|
| **Idle** | Ready to start | "00:00" with "Tap to start" |
| **Working** | Work session active | Red progress ring with countdown |
| **Work Paused** | Work session paused | Gray ring with "Resume" button |
| **Short Break** | 5-minute break | Green progress ring |
| **Long Break** | 15-minute break | Blue progress ring |
| **Paused** | Any paused state | Gray with resume option |

## Project Structure

```
pomodoro-timer/
├── src/
│   ├── main.rs           # Entry point and window setup
│   ├── app.rs            # Main app logic and event handling
│   ├── state.rs          # State machine (TimerState, SessionInfo)
│   ├── timer.rs          # Timer countdown logic with tokio
│   ├── config.rs         # Configuration management
│   ├── persistence.rs    # Save/load timer state
│   ├── notifications.rs  # Desktop notifications
│   └── ui/
│       ├── mod.rs
│       └── circular_timer.rs # Circular timer UI component
├── Cargo.toml
└── README.md
```

## Development

### Building

```bash
cargo build                # Debug build
cargo build --release      # Release build (recommended)
cargo run                  # Run debug build
```

### Architecture

- **State Machine**: Clean state transitions (Idle → Working → Paused → Break)
- **Async Timer**: Tokio-based countdown with 1-second ticks
- **Persistence Layer**: JSON-based state storage
- **GPUI Rendering**: Reactive UI updates with GPUI v0.2.0

## Troubleshooting

### Configuration validation failed

Check your config file:
```bash
cat ~/.config/pomodoro-timer/config.toml
```

Ensure all durations are greater than 0.

### State file corrupted

Reset the state:
```bash
rm ~/.local/share/pomodoro-timer/state.json
```

### Notifications not showing

Ensure you have a notification daemon running:
```bash
# Check if notifications work
notify-send "Test" "This is a test notification"
```

## License

MIT

## Credits

- Built with [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) from Zed
- Uses the [Pomodoro Technique](https://francescocirillo.com/pages/pomodoro-technique) by Francesco Cirillo
