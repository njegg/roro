use std::fs::{OpenOptions, self};
use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
// use rusty_audio::Audio;
use serde_derive::Deserialize;

use crate::notify::send_notification;
use crate::ui::UiMessage;

use TimerState::*;

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
    // audio: Audio,
}

#[derive(Deserialize)]
struct TomlTimerConfig {
    timer_config: TimerConfig,
}

#[derive(Deserialize)]
struct TimerConfig {
    work_time: u64,
    break_time: u64,
    long_break_time: u64,
    long_break_interval: u64,
}

impl Timer {
    fn create(config: TimerConfig) -> Timer {
        // let mut audio = Audio::new();

        // audio.add("notification_sound", "sounds/sound.mp3");

        return Timer {
            is_playing: false,
            state: TimerState::Work,
            time_left: Duration::from_mins(config.work_time),
            pomo: 1,

            work_time: config.work_time,
            break_time: config.break_time,
            long_break_time: config.long_break_time,
            long_break_interval: config.long_break_interval,

            command_waiting_for_confirm: None,

            // audio,
        }
    }

    fn from_toml_config(config_path: &str) -> Option<Timer> {
        let config_str = match fs::read_to_string(config_path) {
            Ok(s) => s,
            Err(err) => {
                log_error(err.to_string());
                return None;
            }
        };

        let config: TomlTimerConfig  = match toml::from_str(&config_str) {
            Ok(c) => c,
            Err(err) => {
                log_error(err.to_string());
                return None;
            }
        };

        Some(Timer::create(config.timer_config))
    }

    fn defaults() -> Timer {
        Timer::create(TimerConfig {
            work_time: 50,
            break_time: 10,
            long_break_time: 60,
            long_break_interval: 4,
        })
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

    fn send_state_to_ui(&self, tx_ui: &Sender<UiMessage>) {
        tx_ui.send(UiMessage::Time(self.time_left)).unwrap();
        tx_ui.send(UiMessage::TimerState(self.state, self.pomo)).unwrap();
    }
}


pub fn spawn_timer_thread(rx: Receiver<TimerCommand>, tx_ui: Sender<UiMessage>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        use TimerCommand::*;

        let config_path = "config/config.toml";

        let mut timer = match Timer::from_toml_config(config_path) {
            Some(t) => t,
            None => {
                tx_ui.send(UiMessage::Error(format!("Error reading: '{}', check roro.log", config_path))).unwrap();
                Timer::defaults()
            }
        };

        timer.send_state_to_ui(&tx_ui);

        loop {
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

                        send_notification(timer.state);
                        // timer.audio.play("notification_sound");

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

                                    Exit => {
                                        tx_ui.send(UiMessage::Exit).unwrap();
                                        return;
                                    },

                                    _ => ()
                                }

                                timer.command_waiting_for_confirm = None;
                                tx_ui.send(UiMessage::ShowConfirm(false)).unwrap();
                            }
                        }

                        Exit => {
                            timer.command_waiting_for_confirm = Some(Exit);
                            tx_ui.send(UiMessage::ShowConfirm(true)).unwrap();
                        },
                    }
                }
            }
        }
    })
}


trait DurationExtension {
    fn from_mins(mins: u64) -> Duration;
}

impl DurationExtension for Duration {
    fn from_mins(mins: u64) -> Duration {
        Duration::from_secs(mins) * 60
    }
}

fn log_error(error_message: String) {
    let mut log_file = OpenOptions::new()
        .append(true)
        .open("roro.log")
        .expect("Cannot open log file");

    let now = chrono::Local::now();
    let date_time = now.format("%Y-%m-%d %H:%M:%S");

    let log_message = format!("[{date_time}]: {error_message}\n");

    log_file
        .write(log_message.as_str().as_bytes())
        .expect("Write to log file failed");
}
