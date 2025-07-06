use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEventKind},
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget, WidgetRef},
};

use crate::{
    app::{AppEvent, Screens},
    event::Event,
    models::db::Database,
    screens::Screen,
};

pub struct MyBoxWidget {
    pub title: String,
}

impl WidgetRef for MyBoxWidget {
    #[doc = " Draws the current state of the widget in the given buffer. That is the only method required"]
    #[doc = " to implement a custom widget."]
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title.clone())
            .borders(Borders::ALL);
        block.render(area, buf);
    }
}

pub struct MyTextWidget {
    pub text: String,
}

impl WidgetRef for MyTextWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let paragraph = Paragraph::new(self.text.clone());
        paragraph.render(area, buf);
    }
}

pub struct ViewFeed {
    count: i32,
    database: Database,
}

impl ViewFeed {
    pub fn new(database: Database) -> Self {
        ViewFeed { count: 0, database }
    }
}

impl Screen for ViewFeed {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        app: &crate::app::App,
    ) {
        let paragraph = Paragraph::new(format!("Count: {}", self.count));
        frame.render_widget(paragraph, area);
    }

    fn handle_input(&mut self, event: &crate::event::EventHandler) -> Option<AppEvent> {
        match event.next().unwrap() {
            Event::Tick => None,
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => Some(AppEvent::Quit),
                        KeyCode::Down => {
                            self.count -= 10;
                            println!("{}", self.count);
                            return None;
                        }
                        KeyCode::Char('s') => {
                            return Some(AppEvent::ChangeScreen(Screens::Home));
                        }
                        KeyCode::Up => {
                            self.count += 10;
                            println!("{}", self.count);
                            return None;
                        }
                        _ => None,
                    };
                }
                None
            }
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
        }
    }
}
