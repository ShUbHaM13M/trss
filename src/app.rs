use crate::{
    event::Event,
    models::{
        config::{Keymap, UserConfig},
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
    crossterm::event::{KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    widgets::WidgetRef,
};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};
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

impl AppState {
    pub fn items_for_selected_source(&self) -> Vec<FeedItem> {
        let mut items = match self.selected_source {
            FeedSource::Feed(feed_id) => self
                .feed_items
                .get(&feed_id)
                .map(|collection| collection.items.clone())
                .unwrap_or_default(),
            FeedSource::Favourites => self
                .feed_items
                .values()
                .flat_map(|collection| collection.items.iter())
                .filter(|item| self.favourites.contains(&item.id))
                .cloned()
                .collect(),
            FeedSource::Readlist => self
                .feed_items
                .values()
                .flat_map(|collection| collection.items.iter())
                .filter(|item| self.readlist.contains(&item.id))
                .cloned()
                .collect(),
        };
        items.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
        items
    }
}

#[derive(Clone)]
pub struct AppConfig {
    pub background_sync: bool,
    pub current_theme: Theme,
    pub current_theme_name: String,
    themes: HashMap<String, Theme>,
    pub keymap: Keymap,
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
    OpenFeed(String),
    OpenAddFeedPopup,
    CloseAddFeedPopup,
    StartBackgroundSync,
    ToggleFeedItemFavourite(String),
    ToggleFeedItemReadlist(String),
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
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let database = Database::init().await?;
        let mut feeds = Feed::get_all(&database).await?;

        let mut feed_items: HashMap<i32, FeedItemCollection> = feeds
            .iter()
            .map(|feed| return (feed.id, FeedItemCollection::new()))
            .collect();

        for feed in &mut feeds {
            let feed_id = feed.id;
            let feed_item = FeedItem::get_by_feed_id(feed_id, &database).await?;
            feed.feed_count = feed_item.items.len() as i32;
            feed_items.insert(feed_id, feed_item);
        }

        let favourites = FeedItem::get_favourites_id(&database).await?;
        let readlist = FeedItem::get_readlist_ids(&database).await?;

        let mut screens: HashMap<Screens, Box<dyn Screen>> = HashMap::new();
        screens.insert(
            Screens::Home,
            Box::new(Home::new(database.clone(), &feeds).await),
        );
        screens.insert(Screens::ViewFeed, Box::new(ViewFeed::new()));

        let user_config = match UserConfig::load() {
            Ok(config) => config,
            Err(error) => {
                eprintln!(
                    "Could not load trss_config.json ({error}); using defaults and themes.json if available."
                );
                UserConfig::default()
            }
        };
        let mut themes = Theme::get_all();
        if user_config.themes.is_empty() {
            match Theme::get_user_themes() {
                Ok(legacy_themes) => themes.extend(legacy_themes),
                Err(error) if !error.to_string().contains("No such file") => {
                    eprintln!("Could not load themes.json: {error}");
                }
                Err(_) => {}
            }
        } else {
            themes.extend(user_config.themes);
        }

        let user_settings = Settings::get_settings(&database).await?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        if user_settings.background_sync {
            spawn_update_feeds(feeds.clone(), event_tx.clone());
        }

        Ok(App {
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
                readlist,
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
                keymap: user_config.keymap,
            },

            event_tx,
            event_rx,

            command_tx,
            command_rx,
        })
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
                    AppCommand::OpenFeed(feed_item_id) => {
                        self.state.selected_feed_item_id = Some(feed_item_id);
                        if self.state.selected_feed_item_id.is_some() {
                            self.current_screen = Screens::ViewFeed;
                        }
                    }
                    AppCommand::ToggleFeedItemFavourite(feed_item_id) => {
                        match FeedItem::toggle_favourite(
                            feed_item_id.clone(),
                            &self.services.database,
                        )
                        .await
                        {
                            Ok(is_favourite) => {
                                for collection in self.state.feed_items.values_mut() {
                                    if let Some(&index) = collection.index_map.get(&feed_item_id) {
                                        collection.items[index].is_favourite = is_favourite;
                                    }
                                }
                                if is_favourite {
                                    self.state.favourites.insert(feed_item_id);
                                    self.state.status_message = Some("Added to favourites".into());
                                } else {
                                    self.state.favourites.remove(&feed_item_id);
                                    self.state.status_message =
                                        Some("Removed from favourites".into());
                                }
                            }
                            Err(error) => {
                                self.state.status_message =
                                    Some(format!("Could not update favourite: {error}"));
                            }
                        }
                    }
                    AppCommand::ToggleFeedItemReadlist(feed_item_id) => {
                        match FeedItem::toggle_readlist(
                            feed_item_id.clone(),
                            &self.services.database,
                        )
                        .await
                        {
                            Ok(in_readlist) => {
                                for collection in self.state.feed_items.values_mut() {
                                    if let Some(&index) = collection.index_map.get(&feed_item_id) {
                                        collection.items[index].in_readlist = in_readlist;
                                    }
                                }
                                if in_readlist {
                                    self.state.readlist.insert(feed_item_id);
                                    self.state.status_message =
                                        Some("Added to reading list".into());
                                } else {
                                    self.state.readlist.remove(&feed_item_id);
                                    self.state.status_message =
                                        Some("Removed from reading list".into());
                                }
                            }
                            Err(error) => {
                                self.state.status_message =
                                    Some(format!("Could not update reading list: {error}"));
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
                            let feed_id = feed.id;
                            let feed_title = feed.title.clone();
                            match Feed::delete(feed_id, &self.services.database).await {
                                Ok(_) => {
                                    self.state.feed_items.remove(&feed_id);
                                    self.state.feeds.remove(self.state.selected_feed_index);
                                    self.state.selected_feed_index = 0;
                                    self.state.selected_source = FeedSource::Feed(
                                        self.state.feeds.first().map_or(0, |feed| feed.id),
                                    );
                                    self.state.status_message =
                                        Some(format!("Removed feed: {feed_title}"));
                                }
                                Err(err) => {
                                    self.state.status_message =
                                        Some(format!("Could not remove {feed_title}: {err}"));
                                }
                            }
                        }
                    }
                }
            }
            if self.should_quit {
                break;
            }

            // Draining Events
            while let Ok(event) = self.event_rx.try_recv() {
                match event {
                    AppEvent::FeedAdded(feed) => {
                        let feed_id = feed.id;
                        let (items, load_error) = match FeedItem::get_by_feed_id(
                            feed_id,
                            &self.services.database,
                        )
                        .await
                        {
                            Ok(items) => (items, None),
                            Err(error) => (FeedItemCollection::new(), Some(error.to_string())),
                        };
                        self.state.feed_items.insert(feed_id, items);
                        let title = feed.title.clone();
                        self.state.feeds.insert(0, feed);
                        self.state.selected_feed_index = 0;
                        self.state.selected_source = FeedSource::Feed(feed_id);
                        self.state.status_message = Some(match load_error {
                            Some(error) => {
                                format!("Added {title}, but could not load its articles: {error}")
                            }
                            None => format!("Added feed: {title}"),
                        });
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

            let event = events.next()?;
            if let Event::Error(message) = &event {
                return Err(std::io::Error::other(message.clone()).into());
            }

            if matches!(&event, Event::Key(key) if key.kind == KeyEventKind::Press) {
                self.state.status_message = None;
            }

            match event {
                Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.state.overlay == Overlay::None {
                            self.state.overlay = Overlay::ThemeSelector;
                            continue;
                        }
                    }
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
                            match Settings::update_setting(
                                &self.services.database,
                                String::from("theme"),
                                self.config.current_theme_name.clone(),
                            )
                            .await
                            {
                                Ok(()) => {
                                    self.state.status_message = Some(format!(
                                        "Theme set to {}",
                                        self.config.current_theme_name
                                    ));
                                }
                                Err(error) => {
                                    self.state.status_message =
                                        Some(format!("Could not save theme: {error}"));
                                }
                            }
                            self.state.overlay = Overlay::None;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if themes.is_empty() {
                                return;
                            }
                            let index = themes
                                .iter()
                                .position(|s| *s == self.config.current_theme_name)
                                .unwrap_or(0);
                            self.config.current_theme_name =
                                themes[(index + 1) % themes.len()].clone();
                            self.config.update_theme();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if themes.is_empty() {
                                return;
                            }
                            let index = themes
                                .iter()
                                .position(|s| *s == self.config.current_theme_name)
                                .unwrap_or(0);
                            let index = if index == 0 {
                                themes.len() - 1
                            } else {
                                index - 1
                            };
                            self.config.current_theme_name = themes[index].clone();
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
