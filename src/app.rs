use crate::{
    models::{db::Database, feed_item::FeedItem},
    screens::{Screen, home::Home, test_html::TestHtml, view_feed::ViewFeed},
};
use ratatui::DefaultTerminal;
use std::collections::HashMap;

use crate::event::EventHandler;

#[derive(Clone)]
pub struct AppState {
    pub selected_feed_item: Option<FeedItem>,
}

pub enum AppEvent {
    Quit,
    ChangeScreen(Screens, AppState),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Screens {
    Home,
    ViewFeed,
    TestHtml,
}

pub struct App {
    current_screen: Screens,
    should_quit: bool,
    screens: HashMap<Screens, Box<dyn Screen>>,
    database: Database,
    pub state: AppState,
}

impl App {
    pub async fn new() -> Self {
        let database = Database::init().await;
        if let Err(err) = database {
            eprintln!("Failed to initialize database: {}", err);
            std::process::exit(1);
        }
        let database = database.unwrap();

        let mut screens: HashMap<Screens, Box<dyn Screen>> = HashMap::new();
        screens.insert(Screens::Home, Box::new(Home::new(database.clone()).await));
        screens.insert(Screens::ViewFeed, Box::new(ViewFeed::new(database.clone())));
        screens.insert(Screens::TestHtml, Box::new(TestHtml::new()));

        App {
            current_screen: Screens::Home,
            // current_screen: Screens::TestHtml,
            should_quit: false,
            screens,
            database,
            state: AppState {
                selected_feed_item: None,
            },
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        let events = EventHandler::new(100);

        while !self.should_quit {
            let current_screen = self.current_screen.clone();
            terminal.draw(|frame| {
                if let Some(screen) = self.screens.get(&current_screen) {
                    screen.render(frame, frame.area(), &self);
                }
            })?;
            if let Some(screen) = self.screens.get_mut(&self.current_screen) {
                if let Some(app_event) = screen.handle_input(&events, &self.state) {
                    match app_event {
                        AppEvent::Quit => self.should_quit = true,
                        AppEvent::ChangeScreen(new_screen, new_state) => {
                            self.state = new_state;
                            self.set_screen(new_screen);
                        }
                    }
                }
            }
            // screen.as_mut().unwrap().handle_input(self, &events);
        }
        Ok(())
    }

    pub fn set_screen(&mut self, screen: Screens) {
        self.current_screen = screen;
        if let Some(screen) = self.screens.get_mut(&self.current_screen) {
            screen.reset();
        }
    }
}
