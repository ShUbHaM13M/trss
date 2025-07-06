use crate::{
    models::db::Database,
    screens::{Screen, home::Home, view_feed::ViewFeed},
};
use ratatui::DefaultTerminal;
use std::collections::HashMap;

use crate::event::EventHandler;

pub enum AppEvent {
    Quit,
    ChangeScreen(Screens),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Screens {
    Home,
    ViewFeed,
}

pub struct App {
    current_screen: Screens,
    should_quit: bool,
    screens: HashMap<Screens, Box<dyn Screen>>,
    database: Database,
}

impl App {
    pub async fn new() -> Self {
        let database = Database::init().await;
        if let Err(err) = database {
            eprintln!("Failed to initialize database: {}", err);
            std::process::exit(1);
        }
        eprintln!("Database initialized successfully");
        let database = database.unwrap();

        let mut screens: HashMap<Screens, Box<dyn Screen>> = HashMap::new();
        screens.insert(Screens::Home, Box::new(Home::new(database.clone()).await));
        screens.insert(Screens::ViewFeed, Box::new(ViewFeed::new(database.clone())));

        App {
            current_screen: Screens::Home,
            should_quit: false,
            screens,
            database,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        let events = EventHandler::new(100);

        while !self.should_quit {
            let current_screen = self.current_screen.clone();
            // let mut screen: Option<Box<dyn Screen>> = match self.current_screen {
            //     Screens::Home => Some(Box::new(Home::new().await)),
            //     Screens::ViewFeed => Some(Box::new(ViewFeed::new())),
            // };
            terminal.draw(|frame| {
                if let Some(screen) = self.screens.get(&current_screen) {
                    screen.render(frame, frame.area(), &self);
                }
                // screen.as_ref().unwrap().render(frame, frame.area(), &self);
                // if screen.is_some() {
                //     screen.as_ref().unwrap().render(frame, frame.area(), &self);
                // } else {
                //     let greeting = Paragraph::new("Hello Ratatui! {} (press 'q' to quit)")
                //         .white()
                //         .on_blue();
                //     frame.render_widget(greeting, frame.area());
                // }
            })?;
            if let Some(screen) = self.screens.get_mut(&self.current_screen) {
                if let Some(app_event) = screen.handle_input(&events) {
                    match app_event {
                        AppEvent::Quit => self.should_quit = true,
                        AppEvent::ChangeScreen(new_screen) => {
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
    }
}
