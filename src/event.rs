use ratatui::crossterm::event::{self, KeyEvent, MouseEvent};
use std::{
    sync::mpsc::{self, RecvError},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct EventHandler(
    mpsc::Sender<Event>,
    mpsc::Receiver<Event>,
    thread::JoinHandle<()>,
);

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

                    if event::poll(timeout).expect("Unable to poll events") {
                        match event::read().expect("Unable to read event") {
                            event::Event::FocusGained => Ok(()),
                            event::Event::FocusLost => Ok(()),
                            event::Event::Key(e) => {
                                if e.kind == event::KeyEventKind::Press {
                                    sender.send(Event::Key(e))
                                } else {
                                    Ok(())
                                }
                            }
                            event::Event::Mouse(e) => sender.send(Event::Mouse(e)),
                            event::Event::Paste(_) => Ok(()),
                            event::Event::Resize(w, h) => sender.send(Event::Resize(w, h)),
                        }
                        .expect("Failed to send terminal events");
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(Event::Tick).expect("Failed to send tick event");
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self(sender, receiver, handler)
    }

    pub fn next(&self) -> Result<Event, RecvError> {
        Ok(self.1.recv()?)
    }
}
