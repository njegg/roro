use std::{io::Write, thread::JoinHandle};
use console::{Key, Term};
use std::{time::Duration, sync::mpsc::Receiver};

use crate::timer::TimerState;


const STATE_Y: usize = 2;
const TIMER_Y: usize = 4;
const POMO_Y: usize = 6;

// Not used yet
const _MIN_WIDTH: u16 = 16;
const _MIN_HEIGHT: u16 = 16;
const _BINDS_Y: usize = 8;
const _CONFIRM_Y: usize = 10;
const _HELP_Y: usize = _MIN_HEIGHT as usize - 1;

pub enum UiMessage {
    Input(Key),
    Time(Duration),
    TimerState(TimerState, u64),
    Error(String),

    /// bool - show or hide the confirmation message
    ShowConfirm(bool), 

    Exit,
}


pub fn spawn_ui_thread(rx: Receiver<UiMessage>) -> JoinHandle<std::io::Result<()>> {
    std::thread::spawn(move || -> std::io::Result<()> {
        use UiMessage::*;

        let mut term = Term::stdout();

        term.hide_cursor()?;
        term.clear_screen()?;

        term.flush()?;
        
        loop {
            match rx.recv().unwrap() {
                Input(key) => {
                    term.println_to(0, term.size().0 as usize - 1, format!("{:?}", key))?;
                }

                Time(time_left) => {
                    let secs_left = time_left.as_secs();

                    let timer_msg = format!(
                        "{:0>2}:{:0>2}",
                        secs_left / 60,
                        secs_left % 60
                    );

                    term.println_to_centered(TIMER_Y, timer_msg)?;
                }

                TimerState(timer_state, pomo) => {
                    term.println_to_centered(STATE_Y, format!("{}", timer_state))?;
                    term.println_to_centered(POMO_Y, format!("pomo #{}", pomo as u8))?;
                }

                ShowConfirm(true) => {
                    term.println_to_centered(_CONFIRM_Y, String::from("[y] to confirm"))?;
                }

                ShowConfirm(false) => {
                    term.move_cursor_to(0, _CONFIRM_Y)?;
                    term.clear_line()?;
                }
                
                Error(message) => {
                    let y = term.size().0 as usize - 1;

                    term.println_to_centered(y, message)?;
                }

                Exit => break,
            }
        }
        
        term.clear_screen()?;
        term.flush()?;
        term.show_cursor()?;

        Ok(())
    })
}


trait TermExtension {
    fn println_to(&mut self, x: usize, y: usize, str: String) -> std::io::Result<()>;
    fn println_to_centered(&mut self, y: usize, str: String) -> std::io::Result<()>;
    fn alt_buffer_enter(&mut self);
    fn alt_buffer_exit(&mut self);
    fn is_too_small(&self, w: u16, h: u16) -> bool;
}

impl TermExtension for Term {
    fn println_to(&mut self, x: usize, y: usize, str: String) -> std::io::Result<()> {
        self.move_cursor_to(x, y)?;
        self.clear_line()?;
        self.move_cursor_to(x, y)?;
        self.write(str.as_bytes())?;
        self.flush()?;

        Ok(())
    }

    fn println_to_centered(&mut self, y: usize, str: String) -> std::io::Result<()> {
        let x = self.size().1 as usize / 2 - str.len() / 2;
        self.println_to(x, y, str)?;

        Ok(())
    }

    fn alt_buffer_enter(&mut self) {
        self.write("\u{1b}[?1049h\x1b[H".as_bytes()).unwrap();
        self.flush().unwrap();
    }

    fn alt_buffer_exit(&mut self) {
        self.write("\u{1b}?1049l".as_bytes()).unwrap();
        self.flush().unwrap();
    }
    
    fn is_too_small(&self, desired_width: u16, desired_height: u16) -> bool {
        let (current_height, current_width) = self.size();
        current_height < desired_height || current_width < desired_width
    }
}
