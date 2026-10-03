use ratatui::crossterm::event::{self, KeyEvent, MouseEvent};
use std::{
    sync::mpsc::{self, RecvError},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Error(String),
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    let has_event = match event::poll(timeout) {
                        Ok(has_event) => has_event,
                        Err(error) => {
                            let _ = sender.send(Event::Error(format!(
                                "Could not poll terminal input: {error}"
                            )));
                            break;
                        }
                    };

                    if has_event {
                        let terminal_event = match event::read() {
                            Ok(terminal_event) => terminal_event,
                            Err(error) => {
                                let _ = sender.send(Event::Error(format!(
                                    "Could not read terminal input: {error}"
                                )));
                                break;
                            }
                        };
                        let app_event = match terminal_event {
                            event::Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                                Some(Event::Key(key))
                            }
                            event::Event::Mouse(mouse) => Some(Event::Mouse(mouse)),
                            event::Event::Resize(width, height) => {
                                Some(Event::Resize(width, height))
                            }
                            event::Event::FocusGained
                            | event::Event::FocusLost
                            | event::Event::Key(_)
                            | event::Event::Paste(_) => None,
                        };
                        if let Some(app_event) = app_event
                            && sender.send(app_event).is_err()
                        {
                            break;
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if sender.send(Event::Tick).is_err() {
                            break;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            receiver,
            _handler: handler,
        }
    }

    pub fn next(&self) -> Result<Event, RecvError> {
        self.receiver.recv()
    }
}
