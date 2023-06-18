use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use crate::ui::UiMessage;


const SECOND: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub enum TimerCommand {
    Play,
    Next,
    Prev,
    Exit,
}

#[derive(Debug)]
#[allow(dead_code)]
enum TimerState {
    Work,
    Break,
    LongBreak
}

#[allow(dead_code)]
struct Timer {
    is_playing: bool,
    state: TimerState,
    time_left: Duration,
    pomo: u32,

    work_time: u32,
    break_time: u32,
    long_break_time: u32,

    long_break_interval: u32,
}

impl Timer {
    fn default_state() -> Timer {
        return Timer {
            is_playing: false,
            state: TimerState::Work,
            time_left: Duration::from_secs(10),
            pomo: 1,

            work_time: 10,
            break_time: 5,
            long_break_time: 15,
            
            long_break_interval: 3,
        }
    }
}


pub fn spawn_timer_thread(rx: Receiver<TimerCommand>, tx_ui: Sender<UiMessage>) {
    std::thread::spawn(move || {
        use TimerCommand::*;

        let mut timer = Timer::default_state();

        tx_ui.send(UiMessage::Time(timer.time_left)).unwrap();

        // TODO is zero => stopped, add Reset state
        while !timer.time_left.is_zero() {
            if !timer.is_playing {
                loop {
                    match rx.recv().unwrap() {
                        Play => { timer.is_playing = true; break },

                        Exit => break,
                        _ => (), // send to ui to update state on Next and Prev
                    }
                }
            } else {
                tx_ui.send(UiMessage::Time(timer.time_left)).unwrap();

                timer.time_left -= SECOND;
                std::thread::sleep(SECOND);

                match rx.try_recv() {
                    Ok(command) => match (&timer.state, &command) {
                        (_, Play) => timer.is_playing = !timer.is_playing,

                        _ => println!("({:?}, {:?})", &timer.state, &command),
                    },
                    Err(why) => match why {
                        mpsc::TryRecvError::Empty => (),
                        mpsc::TryRecvError::Disconnected => panic!("{}", why),
                    }
                }
            }
        }

        tx_ui.send(UiMessage::Time(timer.time_left))
            .expect("timer send failed");
    });

}
