use std::io::Write;
use console::{Key, Term};
use std::{time::Duration, sync::mpsc::Receiver};

use crate::timer::TimerState;


pub enum UiMessage {
    Input(Key),
    Time(Duration),
    TimerState(TimerState),
    Stop,
}


pub fn spawn_ui_thread(rx: Receiver<UiMessage>) {
    std::thread::spawn(move || -> std::io::Result<()> {
        let mut term = Term::stdout();

        term.hide_cursor()?;
        term.clear_screen()?;
        term.clean_write_line_to(0, 6, String::from("pog timer"))?;
        term.clean_write_line_to(0, 0, format!("{}", TimerState::Work))?;
        term.flush()?;
        

        loop {
            match rx.recv().unwrap() {
                UiMessage::Input(key) => {
                    term.clean_write_line_to(0, 2, format!("{:?}", key))?;
                }

                UiMessage::Time(time_left) => {
                    let secs_left = time_left.as_secs();

                    let timer_msg = format!(
                        "time left: {}:{:0}",
                        secs_left / 60,
                        secs_left % 60
                    );

                    term.clean_write_line_to(0, 1, timer_msg)?;
                }

                UiMessage::TimerState(timer_state) => {
                    term.clean_write_line_to(0, 0, format!("{}", timer_state))?;
                }

                UiMessage::Stop => break,
            }
        }
        
        term.clear_screen()?;
        term.show_cursor()?;
        term.flush()?;

        Ok(())
    });
    
}


trait TermExtension {
    fn clean_write_line_to(&mut self, x: usize, y: usize, str: String) -> std::io::Result<()>;
    fn alt_buffer_enter(&mut self);
    fn alt_buffer_exit(&mut self);
}

impl TermExtension for Term {
    fn clean_write_line_to(&mut self, x: usize, y: usize, str: String) -> std::io::Result<()> {
        self.move_cursor_to(x, y)?;
        self.clear_line()?;
        self.write(str.as_bytes())?;
        self.flush()?;

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
}
