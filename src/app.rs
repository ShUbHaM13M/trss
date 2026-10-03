use crate::{
    event::Event,
    models::{
        db::Database,
        feed::{Feed, FeedSource, spawn_update_feeds},
        feed_item::{FeedItem, FeedItemCollection},
        settings::Settings,
        theme::{DARK_THEME, Theme},
    },
    screens::{Screen, ScreenContext, ScreenContextMut, home::Home, view_feed::ViewFeed},
    widgets::{footer::Footer, theme_selector::ThemeSelector},
};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    widgets::WidgetRef,
};
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;

use crate::event::EventHandler;

#[derive(Debug)]
pub enum AppEvent {
    FeedAdded(Feed),
    FeedAddFailed(String),
    BackgroundSyncStarted,
    BackgroundSyncFinished,
    FeedsUpdated(Vec<i32>, Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Overlay {
    None,
    ThemeSelector,
    AddFeedPopup,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub feeds: Vec<Feed>,
    pub feed_items: HashMap<i32, FeedItemCollection>,
    pub favourites: HashSet<String>,
    pub readlist: HashSet<String>,
    pub selected_source: FeedSource,
    pub selected_feed_index: usize,
    pub selected_feed_item_id: Option<String>,
    pub background_syncing: bool,
    pub overlay: Overlay,
    pub status_message: Option<String>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub background_sync: bool,
    pub current_theme: Theme,
    pub current_theme_name: String,
    themes: HashMap<String, Theme>,
}

impl AppConfig {
    pub fn set_theme(&mut self, theme: Theme) {
        self.current_theme = theme;
    }
    pub fn update_theme(&mut self) {
        let theme = self.themes.get(&self.current_theme_name);
        self.set_theme(theme.unwrap_or(&DARK_THEME).clone());
    }
}

pub struct Services {
    pub database: Database,
}

#[derive(Debug)]
pub enum AppCommand {
    Quit,
    Navigate(Screens, Option<AppState>),
    OpenFeed(usize),
    OpenAddFeedPopup,
    CloseAddFeedPopup,
    StartBackgroundSync,
    ToggleFeedItemFavourite(String),
    // SelectFeed(usize),
    SelectSource(FeedSource),
    DeleteSelectedFeed,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Screens {
    Home,
    ViewFeed,
    TestHtml,
}

pub struct App {
    current_screen: Screens,
    should_quit: bool,

    screens: HashMap<Screens, Box<dyn Screen>>,

    pub state: AppState,
    pub config: AppConfig,
    pub services: Services,

    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: mpsc::UnboundedReceiver<AppEvent>,

    command_tx: mpsc::UnboundedSender<AppCommand>,
    command_rx: mpsc::UnboundedReceiver<AppCommand>,
}

impl App {
    pub async fn new() -> Self {
        let database = Database::init().await;
        if let Err(err) = database {
            eprintln!("Failed to initialize database: {}", err);
            std::process::exit(1);
        }
        let database = database.unwrap();

        let mut feeds: Vec<Feed> = Vec::new();
        if let Ok(f) = Feed::get_all(&database).await {
            feeds = f;
        }

        let mut feed_items: HashMap<i32, FeedItemCollection> = feeds
            .iter()
            .map(|feed| return (feed.id, FeedItemCollection::new()))
            .collect();

        for feed in &mut feeds {
            let feed_id = feed.id;
            match FeedItem::get_by_feed_id(feed_id, &database).await {
                Ok(feed_item) => {
                    let count = feed_item.items.len() as i32;
                    feed.feed_count = count;
                    feed_items.insert(feed_id, feed_item);
                }
                Err(e) => panic!("Failed to fetch feed items for feed id {}", e),
            };
        }

        let favourites = match FeedItem::get_favourites_id(&database).await {
            Ok(favourites) => favourites,
            Err(_) => HashSet::new(),
        };

        let mut screens: HashMap<Screens, Box<dyn Screen>> = HashMap::new();
        screens.insert(
            Screens::Home,
            Box::new(Home::new(database.clone(), &feeds).await),
        );
        screens.insert(Screens::ViewFeed, Box::new(ViewFeed::new(database.clone())));

        let mut themes = Theme::get_all();
        let user_themes = Theme::get_user_themes().unwrap_or_default();
        themes.extend(user_themes);

        let user_settings = Settings::get_settings(&database).await;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        if user_settings.background_sync {
            spawn_update_feeds(feeds.clone(), event_tx.clone());
        }

        App {
            current_screen: Screens::Home,
            should_quit: false,

            screens,

            services: Services {
                database: database.clone(),
            },
            state: AppState {
                feed_items,
                // Feed id 0 is a harmless empty-state selection until the user adds
                // their first feed (database feed ids start at 1).
                selected_source: FeedSource::Feed(feeds.first().map_or(0, |feed| feed.id)),
                feeds,
                selected_feed_index: 0,
                selected_feed_item_id: None,
                background_syncing: false,
                overlay: Overlay::None,
                favourites,
                readlist: HashSet::new(),
                status_message: None,
            },
            config: AppConfig {
                background_sync: user_settings.background_sync,
                current_theme: themes
                    .get(&user_settings.theme)
                    .unwrap_or(&DARK_THEME)
                    .clone(),
                current_theme_name: user_settings.theme,
                themes,
            },

            event_tx,
            event_rx,

            command_tx,
            command_rx,
        }
    }

    pub async fn run(
        &mut self,
        mut terminal: DefaultTerminal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut themes: Vec<String> = self
            .config
            .themes
            .keys()
            .map(|name| name.to_string())
            .collect();
        themes.sort();

        let events = EventHandler::new(500);

        while !self.should_quit {
            // Draining Commands
            while let Ok(cmd) = self.command_rx.try_recv() {
                match cmd {
                    AppCommand::Navigate(screen, _) => {
                        self.current_screen = screen;
                    }
                    AppCommand::StartBackgroundSync => {
                        if !self.state.background_syncing {
                            self.state.background_syncing = true;
                            spawn_update_feeds(self.state.feeds.clone(), self.event_tx.clone());
                        }
                    }
                    AppCommand::Quit => {
                        self.should_quit = true;
                    }
                    AppCommand::OpenAddFeedPopup => {
                        self.state.overlay = Overlay::AddFeedPopup;
                    }
                    AppCommand::CloseAddFeedPopup => {
                        self.state.overlay = Overlay::None;
                    }
                    AppCommand::OpenFeed(index) => {
                        match self.state.selected_source {
                            FeedSource::Favourites => {
                                self.state.selected_feed_item_id =
                                    self.state.favourites.iter().nth(index).cloned();
                            }
                            FeedSource::Readlist => {
                                self.state.selected_feed_item_id =
                                    self.state.readlist.iter().nth(index).cloned();
                            }
                            FeedSource::Feed(id) => {
                                self.state.selected_feed_item_id = self
                                    .state
                                    .feed_items
                                    .get(&id)
                                    .and_then(|collection| collection.items.get(index))
                                    .map(|item| item.id.clone());
                            }
                        }
                        if self.state.selected_feed_item_id.is_some() {
                            self.current_screen = Screens::ViewFeed;
                        }
                    }
                    AppCommand::ToggleFeedItemFavourite(feed_item_id) => {
                        let is_favourite = FeedItem::toggle_favourite(
                            feed_item_id.clone(),
                            &self.services.database.clone(),
                        )
                        .await;
                        if let Some(feed) = self.state.feeds.get(self.state.selected_feed_index) {
                            if let Some(collection) = self.state.feed_items.get_mut(&feed.id) {
                                if let Some(&index) = collection.index_map.get(&feed_item_id) {
                                    collection.items[index].is_favourite = is_favourite;
                                    if is_favourite {
                                        self.state.favourites.insert(feed_item_id.clone());
                                    } else {
                                        self.state.favourites.remove(&feed_item_id);
                                    }
                                }
                            }
                        }
                    }
                    AppCommand::SelectSource(source) => {
                        self.state.selected_source = source.clone();
                        if let FeedSource::Feed(id) = source {
                            if let Some(index) = self.state.feeds.iter().position(|f| f.id == id) {
                                self.state.selected_feed_index = index;
                            }
                        }
                        // self.state.selected_feed_index = index;
                    }
                    AppCommand::DeleteSelectedFeed => {
                        if let Some(feed) = self.state.feeds.get(self.state.selected_feed_index) {
                            match Feed::delete(feed.id, &self.services.database).await {
                                Ok(_) => {
                                    self.state.feed_items.remove(&feed.id);
                                    self.state.feeds.remove(self.state.selected_feed_index);
                                    self.state.selected_feed_index = 0;
                                }
                                Err(err) => {
                                    self.state.status_message =
                                        Some(format!("Could not remove {}: {err}", feed.title));
                                }
                            }
                        }
                    }
                }
            }

            // Draining Events
            while let Ok(event) = self.event_rx.try_recv() {
                match event {
                    AppEvent::FeedAdded(feed) => {
                        let feed_id = feed.id;
                        self.state.feed_items.insert(
                            feed_id,
                            match FeedItem::get_by_feed_id(feed_id, &self.services.database).await {
                                Ok(item) => item,
                                Err(_) => FeedItemCollection::new(),
                            },
                        );
                        self.state.feeds.insert(0, feed);
                        self.state.selected_feed_index = 0;
                        self.state.selected_source = FeedSource::Feed(feed_id);
                        self.state.status_message =
                            Some(format!("Added feed: {}", self.state.feeds[0].title));
                    }
                    AppEvent::FeedAddFailed(err) => {
                        self.state.status_message = Some(format!("Could not add feed: {err}"));
                    }
                    AppEvent::BackgroundSyncStarted => {
                        self.state.background_syncing = true;
                        self.state.status_message = Some("Refreshing feeds…".to_string());
                    }
                    AppEvent::BackgroundSyncFinished => {
                        self.state.background_syncing = false;
                    }
                    AppEvent::FeedsUpdated(updated_ids, mut errors) => {
                        self.state.background_syncing = false;
                        let selected_item_id = self.state.selected_feed_item_id.clone();
                        let selected_item_was_in_updated_feed =
                            selected_item_id.as_ref().is_some_and(|id| {
                                updated_ids.iter().any(|feed_id| {
                                    self.state
                                        .feed_items
                                        .get(feed_id)
                                        .is_some_and(|collection| {
                                            collection.index_map.contains_key(id)
                                        })
                                })
                            });
                        for feed_id in &updated_ids {
                            match FeedItem::get_by_feed_id(*feed_id, &self.services.database).await
                            {
                                Ok(collection) => {
                                    if let Some(feed) =
                                        self.state.feeds.iter_mut().find(|feed| feed.id == *feed_id)
                                    {
                                        feed.feed_count = collection.items.len() as i32;
                                    }
                                    self.state.feed_items.insert(*feed_id, collection);
                                }
                                Err(error) => {
                                    errors.push(format!("Could not reload feed {feed_id}: {error}"))
                                }
                            }
                        }
                        if let Ok(favourites) =
                            FeedItem::get_favourites_id(&self.services.database).await
                        {
                            self.state.favourites = favourites;
                        }
                        if !updated_ids.is_empty() {
                            if let Ok(feeds) = Feed::get_all(&self.services.database).await {
                                for refreshed in feeds {
                                    if let Some(current) = self
                                        .state
                                        .feeds
                                        .iter_mut()
                                        .find(|feed| feed.id == refreshed.id)
                                    {
                                        current.title = refreshed.title;
                                        current.subtitle = refreshed.subtitle;
                                        current.url = refreshed.url;
                                    }
                                }
                            }
                        }
                        if selected_item_was_in_updated_feed {
                            let selected_item_still_exists =
                                selected_item_id.as_ref().is_some_and(|id| {
                                    self.state
                                        .feed_items
                                        .values()
                                        .any(|collection| collection.index_map.contains_key(id))
                                });
                            if let Some(view_feed) = self.screens.get_mut(&Screens::ViewFeed) {
                                view_feed.reset();
                            }
                            if !selected_item_still_exists {
                                self.state.selected_feed_item_id = None;
                                self.current_screen = Screens::Home;
                            }
                        }
                        self.state.status_message = Some(if errors.is_empty() {
                            format!("Refresh complete: {} feed(s) updated", updated_ids.len())
                        } else {
                            let details = errors
                                .iter()
                                .take(3)
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("; ");
                            let remaining = errors.len().saturating_sub(3);
                            if remaining == 0 {
                                format!("Refresh finished with errors: {details}")
                            } else {
                                format!(
                                    "Refresh finished with errors: {details}; and {remaining} more"
                                )
                            }
                        });
                    }
                }
            }

            // Rendering
            terminal.draw(|frame| {
                let [body, footer_area] = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Fill(1), Constraint::Max(3)])
                    .areas(frame.area());

                let ctx = ScreenContext {
                    state: &self.state,
                    config: &self.config,
                };

                self.screens[&self.current_screen].render(frame, body, &ctx);
                let footer = Footer {
                    current_screen: self.current_screen.clone(),
                    theme: self.config.current_theme,
                    message: self.state.status_message.as_deref(),
                };
                footer.render_ref(footer_area, frame.buffer_mut());

                if self.state.overlay == Overlay::ThemeSelector {
                    let theme_selector = ThemeSelector::new(
                        &themes,
                        self.config.current_theme,
                        themes
                            .iter()
                            .position(|s| *s == self.config.current_theme_name),
                    );
                    theme_selector.render_ref(frame.area(), frame.buffer_mut());
                }
            })?;

            let event = events.next().unwrap();

            if matches!(&event, Event::Key(key) if key.kind == KeyEventKind::Press) {
                self.state.status_message = None;
            }

            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('t') => {
                        if self.state.overlay == Overlay::None {
                            self.state.overlay = Overlay::ThemeSelector;
                            continue;
                        }
                    }
                    KeyCode::Esc => match self.state.overlay {
                        Overlay::None => {
                            self.should_quit = true;
                            continue;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }

            match self.state.overlay {
                Overlay::None => {
                    let screen = self.screens.get_mut(&self.current_screen);
                    if let Some(screen) = screen {
                        screen.handle_input(
                            &event,
                            &ScreenContextMut {
                                state: &mut self.state,
                                config: &mut self.config,
                                event_tx: self.event_tx.clone(),
                                command_tx: self.command_tx.clone(),
                            },
                        );
                    }
                }
                Overlay::ThemeSelector => {
                    self.handle_theme_selector_events(&event, &themes).await;
                }
                Overlay::AddFeedPopup => {
                    let screen = self.screens.get_mut(&self.current_screen);
                    if let Some(screen) = screen {
                        screen.handle_input(
                            &event,
                            &ScreenContextMut {
                                state: &mut self.state,
                                config: &mut self.config,
                                event_tx: self.event_tx.clone(),
                                command_tx: self.command_tx.clone(),
                            },
                        );
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_theme_selector_events(&mut self, event: &Event, themes: &Vec<String>) {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            self.state.overlay = Overlay::None;
                        }
                        KeyCode::Enter => {
                            let _ = Settings::update_setting(
                                &self.services.database,
                                String::from("theme"),
                                self.config.current_theme_name.clone(),
                            )
                            .await;
                            self.state.overlay = Overlay::None;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let index = themes
                                .iter()
                                .position(|s| *s == self.config.current_theme_name);
                            if index.is_none() {
                                self.config.current_theme_name =
                                    themes.get(index.unwrap()).unwrap().clone();
                            } else {
                                self.config.current_theme_name = themes
                                    .get(index.unwrap().wrapping_add(1) % themes.len())
                                    .unwrap()
                                    .clone();
                            }
                            self.config.update_theme();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let mut index = themes
                                .iter()
                                .position(|s| *s == self.config.current_theme_name)
                                .unwrap_or_default();
                            index = if index == 0 {
                                themes.len() - 1
                            } else {
                                index - 1
                            };
                            self.config.current_theme_name = themes.get(index).unwrap().clone();
                            self.config.update_theme();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
