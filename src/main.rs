pub mod timer;
pub mod ui;
pub mod notify;
pub mod input;

use std::sync::mpsc;
use input::spawn_input_thread;
use timer::{spawn_timer_thread, TimerCommand};
use ui::{spawn_ui_thread, UiMessage};


fn main() -> std::io::Result<()> {
    let (tx_ui, rx_ui) = mpsc::channel::<UiMessage>();
    let (tx_timer, rx_timer) = mpsc::channel::<TimerCommand>();

    let ui_thread_handle = spawn_ui_thread(rx_ui);
    let timer_thread_handle = spawn_timer_thread(rx_timer, tx_ui.clone());

    spawn_input_thread(tx_timer.clone());

    ui_thread_handle.join().unwrap()?;
    timer_thread_handle.join().unwrap();

    Ok(())
}

