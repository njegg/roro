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
    time_left: Duration,
    pomo: u64,

    work_time: u64,
    break_time: u64,
    long_break_time: u64,

    long_break_interval: u64,

    state: TimerState,
    is_playing: bool,
}

impl Timer {
    fn default_state() -> Timer {
        let work_time: u64 = 25;

        return Timer {
            is_playing: false,
            state: TimerState::Work,
            time_left: Duration::from_secs(work_time * 60),
            pomo: 1,

            work_time,
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
            if timer.is_playing {
                std::thread::sleep(SECOND);

                // Check for commands without blocking
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

                // If pause was hit while timer was sleeping dont decrease time
                if timer.is_playing {
                    timer.time_left -= SECOND;
                }

                tx_ui.send(UiMessage::Time(timer.time_left)).unwrap();
            } else {
                loop {
                    // Wait for command
                    match rx.recv().unwrap() {
                        Play => { timer.is_playing = true; break },

                        Exit => break,
                        _ => (), // send ui to update state on Next and Prev
                    }
                }
            }
        }

        tx_ui.send(UiMessage::Time(timer.time_left))
            .expect("timer send failed");
    });

}
