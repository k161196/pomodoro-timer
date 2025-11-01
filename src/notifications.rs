use notify_rust::Notification;

pub fn notify_work_complete() {
    let _ = Notification::new()
        .summary("Work Session Complete!")
        .body("Time for a break. Great job!")
        .timeout(5000)
        .show();
}

pub fn notify_break_complete() {
    let _ = Notification::new()
        .summary("Break Complete!")
        .body("Ready to focus again?")
        .timeout(5000)
        .show();
}

pub fn notify_long_break_complete() {
    let _ = Notification::new()
        .summary("Long Break Complete!")
        .body("You've completed a full Pomodoro cycle. Well done!")
        .timeout(5000)
        .show();
}

pub fn log_info(message: &str) {
    eprintln!("[INFO] {}", message);
}

pub fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}
