use notify_rust::Notification;

pub fn notify_work_complete() {
    log_info("Sending work complete notification...");
    match Notification::new()
        .summary("Work Session Complete!")
        .body("Time for a break. Great job!")
        .timeout(5000)
        .sound_name("message-new-instant")  // System notification sound
        .show() {
            Ok(_) => log_info("Work complete notification sent successfully"),
            Err(e) => log_error(&format!("Failed to send work complete notification: {}", e)),
        }
}

pub fn notify_break_complete() {
    log_info("Sending break complete notification...");
    match Notification::new()
        .summary("Break Complete!")
        .body("Ready to focus again?")
        .timeout(5000)
        .sound_name("message-new-instant")  // System notification sound
        .show() {
            Ok(_) => log_info("Break complete notification sent successfully"),
            Err(e) => log_error(&format!("Failed to send break complete notification: {}", e)),
        }
}

pub fn notify_long_break_complete() {
    log_info("Sending long break complete notification...");
    match Notification::new()
        .summary("Long Break Complete!")
        .body("You've completed a full Pomodoro cycle. Well done!")
        .timeout(5000)
        .sound_name("message-new-instant")  // System notification sound
        .show() {
            Ok(_) => log_info("Long break complete notification sent successfully"),
            Err(e) => log_error(&format!("Failed to send long break complete notification: {}", e)),
        }
}

pub fn log_info(message: &str) {
    eprintln!("[INFO] {}", message);
}

pub fn log_error(message: &str) {
    eprintln!("[ERROR] {}", message);
}
