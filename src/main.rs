pub mod timer;
pub mod ui;
pub mod notify;

use std::sync::mpsc;
use console::{Term, Key};
use timer::{spawn_timer_thread, TimerCommand};
use ui::{spawn_ui_thread, UiMessage};


fn main() -> std::io::Result<()> {
    let term = Term::stdout();

    let (tx_ui, rx_ui) = mpsc::channel::<UiMessage>();
    let (tx_timer, rx_timer) = mpsc::channel::<TimerCommand>();

    let ui_thread_handle = spawn_ui_thread(rx_ui);
    let timer_thread_handle = spawn_timer_thread(rx_timer, tx_ui.clone());

    loop {
        match term.read_key() {
            Ok(key) => match key {
                Key::Escape | Key::Char('q') => break,

                Key::Char(' ') => tx_timer.send(TimerCommand::Play).unwrap(),
                Key::Char('n') => tx_timer.send(TimerCommand::Next).unwrap(),
                Key::Char('p') => tx_timer.send(TimerCommand::Prev).unwrap(),

                Key::Char('y') => tx_timer.send(TimerCommand::Confirm(true)).unwrap(),

                _ => tx_timer.send(TimerCommand::Confirm(false)).unwrap(),
            },
            Err(why) => panic!("{why}")
        }
    }

    tx_ui.send(UiMessage::Exit).unwrap();
    tx_timer.send(TimerCommand::Exit).unwrap();

    ui_thread_handle.join().unwrap()?;
    timer_thread_handle.join().unwrap();

    Ok(())
}

