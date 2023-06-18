pub mod timer;
pub mod ui;

use std::sync::mpsc;
use console::{Term, Key};
use timer::{spawn_timer_thread, TimerCommand};
use ui::{spawn_ui_thread, UiMessage};


fn main() -> std::io::Result<()> {
    let term = Term::stdout();

    let (tx_ui, rx_ui) = mpsc::channel::<UiMessage>();
    let (tx_timer, rx_timer) = mpsc::channel::<TimerCommand>();

    spawn_ui_thread(rx_ui);
    spawn_timer_thread(rx_timer, tx_ui.clone());


    loop {
        match term.read_key() {
            Ok(key) => match key {
                Key::Escape => break,
                Key::Char(' ') => tx_timer.send(TimerCommand::Play).unwrap(),

                _ => tx_ui.send(UiMessage::Input(key)).unwrap()
            },
            Err(why) => panic!("{why}")
        }
    }

    tx_ui.send(UiMessage::Stop).unwrap();
    tx_timer.send(TimerCommand::Exit).unwrap();

    Ok(())
}

