use std::sync::mpsc::Sender;

use console::{Term, Key};

use crate::timer::TimerCommand;


pub fn spawn_input_thread(tx_timer: Sender<TimerCommand>) {
    std::thread::spawn(move || {
        let term = Term::stdout();

        loop {
            match term.read_key() {
                Ok(key) => match key {
                    Key::Escape | Key::Char('q') => tx_timer.send(TimerCommand::Exit).unwrap(),

                    Key::Char(' ') => tx_timer.send(TimerCommand::Play).unwrap(),
                    Key::Char('n') => tx_timer.send(TimerCommand::Next).unwrap(),
                    Key::Char('p') => tx_timer.send(TimerCommand::Prev).unwrap(),

                    Key::Char('y') => tx_timer.send(TimerCommand::Confirm(true)).unwrap(),

                    _ => tx_timer.send(TimerCommand::Confirm(false)).unwrap(),
                },
                Err(why) => panic!("{why}")
            }
        }
    });

    ()
}

