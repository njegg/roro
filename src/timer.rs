use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use TimerState::*;
use notify_rust::{Notification, Timeout};

use crate::ui::UiMessage;


const SECOND: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub enum TimerCommand {
    Play,
    Next,
    Prev,
    Confirm(bool),
    Exit,
}

#[derive(Default, Clone, Copy)]
pub enum TimerState {
    #[default]
    Work,
    Break,
    LongBreak
}

impl std::fmt::Display for TimerState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match *self {
            Work => "Work",
            Break => "Break",
            LongBreak => "Long Break",
        };

        write!(f, "{}", printable)
    }
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
    command_waiting_for_confirm: Option<TimerCommand>,
}

impl Timer {
    fn defaults() -> Timer {
        let work_time: u64 = 30;
        let break_time: u64 = 5;
        let long_break_time: u64 = 60;
        let long_break_interval: u64 = 4;

        return Timer {
            is_playing: false,
            state: TimerState::Work,
            time_left: Duration::from_mins(work_time),
            pomo: 1,

            work_time,
            break_time,
            long_break_time,
            long_break_interval,

            command_waiting_for_confirm: None,
        }
    }

    fn next_state(&mut self) {
        let mut new_pomo_value = self.pomo;

        self.set_state(match self.state {
            Work => {
                if self.pomo % self.long_break_interval == 0 {
                    LongBreak
                } else {
                    Break
                }
            }

            Break | LongBreak => {
                new_pomo_value += 1;
                Work
            }
        });

        self.pomo = new_pomo_value;
    } 

    fn prev_state(&mut self) {
        if self.pomo == 1 {
            self.set_state(Work);
            return;
        }

        let mut new_pomo_value = self.pomo;

        self.set_state(match self.state {
            Work => {
                new_pomo_value -= 1;

                if self.pomo % self.long_break_interval == 1 {
                    LongBreak
                } else {
                    Break
                }
            }

            Break | LongBreak => Work
        });

        self.pomo = new_pomo_value;
    } 

    fn set_state(&mut self, new_state: TimerState) {
        self.state = new_state;
        self.is_playing = false;

        self.time_left = Duration::from_mins(
            match new_state {
                Work => self.work_time,
                Break => self.break_time,
                LongBreak => self.long_break_time 
            }
        );
    }

    pub fn is_zero(&self) -> bool {
        self.time_left.is_zero()
    }

    fn send_state_to_ui(&self, tx_ui: &Sender<UiMessage>) {
        tx_ui.send(UiMessage::Time(self.time_left)).unwrap();
        tx_ui.send(UiMessage::TimerState(self.state, self.pomo)).unwrap();
    }
}


pub fn spawn_timer_thread(rx: Receiver<TimerCommand>, tx_ui: Sender<UiMessage>) {
    std::thread::spawn(move || {
        use TimerCommand::*;

        let mut timer = Timer::defaults();

        timer.send_state_to_ui(&tx_ui);

        while !timer.is_zero() {
            if timer.is_playing {
                std::thread::sleep(SECOND);

                loop { // Check for commands without blocking
                    match rx.try_recv() {
                        Ok(command) => match command {
                            Play  => timer.is_playing = !timer.is_playing,

                            _ => ()
                        },

                        Err(why) => match why {
                            mpsc::TryRecvError::Empty => break,
                            mpsc::TryRecvError::Disconnected => panic!("{}", why),
                        }
                    }
                }

                // If pause was hit while timer was sleeping dont decrease time
                if timer.is_playing {
                    timer.time_left -= SECOND;

                    if timer.time_left.is_zero() {
                        timer.is_playing = false;
                        timer.next_state();


                        let notification = match &timer.state {
                            Work => ("Break done", "Time for work!"),
                            Break => ("Work done", "Time to take a short break!"),
                            LongBreak => ("Work done", "Time for a long break!"),
                        };


                        Notification::new()
                            .summary(notification.0)
                            .body(notification.1)
                            .timeout(Timeout::Never) // this however is
                            .show().unwrap();


                        tx_ui.send(UiMessage::TimerState(timer.state, timer.pomo)).unwrap();
                    }
                }

                tx_ui.send(UiMessage::Time(timer.time_left)).unwrap();
            } else {
                loop {
                    match rx.recv().unwrap() { // Block untill command is recieved
                        Play => { 
                            tx_ui.send(UiMessage::ShowConfirm(false)).unwrap();
                            timer.command_waiting_for_confirm = None;

                            timer.is_playing = true;
                            break
                        }

                        Next => {
                            timer.command_waiting_for_confirm = Some(Next);
                            tx_ui.send(UiMessage::ShowConfirm(true)).unwrap();
                        }

                        Prev => {
                            timer.command_waiting_for_confirm = Some(Prev);
                            tx_ui.send(UiMessage::ShowConfirm(true)).unwrap();
                        }

                        Confirm(confirmed) => {
                            if !confirmed {
                                timer.command_waiting_for_confirm = None;
                                tx_ui.send(UiMessage::ShowConfirm(false)).unwrap();
                            } else if let Some(command) = &timer.command_waiting_for_confirm {
                                match command {
                                    Next => {
                                        timer.next_state();
                                        timer.send_state_to_ui(&tx_ui);
                                    }

                                    Prev => {
                                        timer.prev_state();
                                        timer.send_state_to_ui(&tx_ui);
                                    }

                                    _ => ()
                                }

                                timer.command_waiting_for_confirm = None;
                                tx_ui.send(UiMessage::ShowConfirm(false)).unwrap();
                            }
                        }

                        Exit => break,
                    }
                }
            }
        }

        timer.send_state_to_ui(&tx_ui);
    });
}


trait DurationExtension {
    fn from_mins(mins: u64) -> Duration;
}

impl DurationExtension for Duration {
    fn from_mins(mins: u64) -> Duration {
        Duration::from_secs(mins) * 60
    }
}

