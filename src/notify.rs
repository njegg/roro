use notify_rust::{Notification, Timeout};

use crate::timer::TimerState;


pub fn send_notification(state: TimerState) {
    let message = match state {
        TimerState::Work => ("Break done", "Time for work!"),
        TimerState::Break => ("Work done", "Time to take a short break!"),
        TimerState::LongBreak => ("Work done", "Time for a long break!"),
    };

    Notification::new()
        .summary(message.0)
        .body(message.1)
        .timeout(Timeout::Never)
        .show().unwrap();
}

