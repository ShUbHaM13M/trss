use ratatui::crossterm::event::KeyModifiers;
use ratatui::widgets::WidgetRef;
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Direction, Layout},
    widgets::Block,
};
use ratatui::{prelude::Constraint, style::Stylize};
use tokio::sync::mpsc;

use crate::app::{AppCommand, AppEvent, Screens};
use crate::models::db::Database;
use crate::models::feed::{Feed, FeedSource};
use crate::models::feed_item::FeedItem;
use crate::screens::{ScreenContext, ScreenContextMut};
use crate::widgets::add_feed::AddFeed;
use crate::widgets::feed_list::{FeedList, FeedListView};
use crate::widgets::header::Header;
use crate::widgets::sidebar::{Sidebar, SidebarView};
use crate::{event::Event, screens::Screen};

#[derive(Clone, PartialEq)]
pub enum FocusedWidget {
    Search,
    Categories,
    FeedList,
    FeedItems,
    AddFeed,
}

#[derive(Clone)]
pub struct HomeState {
    pub selected_feed_item_index: Option<usize>,
    // pub selected_navigation_index: usize,
    pub focused_widget: FocusedWidget,
    pub filtered_feeds: Vec<Feed>,
    pub search_query: String,
    pub new_feed_title: String,
    pub new_feed_url: String,
    pub add_feed_index: usize,
}

pub struct Home {
    show_sidebar: bool,
    // update_feeds_receiver: Option<Receiver<Vec<i32>>>,
    state: HomeState,
    database: Database,
}

impl Home {
    pub async fn new(database: Database, feeds: &Vec<Feed>) -> Self {
        // let (_sender, receiver) = channel();
        // if APP_CONFIG.background_sync {
        //     spawn_update_feeds(feeds.clone(), sender).await;
        // }

        let state = HomeState {
            // selected_navigation_index: 0,
            selected_feed_item_index: None,
            focused_widget: FocusedWidget::FeedList,
            filtered_feeds: feeds
                .iter()
                .filter(|f| {
                    let title = f.title.to_lowercase();
                    title != "favourites" && title != "readlist"
                })
                .cloned()
                .collect(),
            search_query: String::new(),
            new_feed_title: String::new(),
            new_feed_url: String::new(),
            add_feed_index: 0,
        };

        Home {
            show_sidebar: true,
            state,
            database,
        }
    }

    pub fn filter_feeds(&mut self, feeds: &[Feed], current_source: &FeedSource) {
        self.state.filtered_feeds = match current_source {
            FeedSource::Feed(_) => feeds
                .iter()
                .filter(|f| {
                    let t = f.title.to_lowercase();
                    t != "favourites" && t != "readlist"
                })
                .cloned()
                .collect(),
            FeedSource::Favourites => feeds
                .iter()
                .filter(|f| f.title.to_lowercase() == "favourites")
                .cloned()
                .collect(),
            FeedSource::Readlist => feeds
                .iter()
                .filter(|f| f.title.to_lowercase() == "readlist")
                .cloned()
                .collect(),
        };

        if !self.state.search_query.is_empty() {
            let query = self.state.search_query.to_lowercase();
            self.state
                .filtered_feeds
                .retain(|f| f.title.to_lowercase().contains(&query));
        }
    }

    fn add_feed(&mut self, tx: mpsc::UnboundedSender<AppEvent>) {
        let title = self.state.new_feed_title.clone();
        let url = self.state.new_feed_url.clone();
        let database = self.database.clone();
        tokio::spawn(async move {
            match Feed::add(title, url, &database).await {
                Ok(feed) => {
                    let _ = tx.send(AppEvent::FeedAdded(feed));
                }
                Err(err) => {
                    let _ = tx.send(AppEvent::FeedAddFailed(err.to_string()));
                }
            }
        });
    }
}

impl Screen for Home {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        ctx: &ScreenContext,
    ) {
        let container = Block::default().bg(ctx.config.current_theme.background);
        let inner = container.inner(area);
        frame.render_widget(container, area);

        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(3), Constraint::Fill(1)])
            .areas(inner);

        // Header
        {
            let header = Header {
                current_screen: Screens::Home,
                theme: ctx.config.current_theme,
                search_query: &self.state.search_query,
                search_focused: self.state.focused_widget == FocusedWidget::Search,
                syncing: ctx.state.background_syncing,
            };

            header.render_ref(header_area, frame.buffer_mut());
        }

        // Content
        {
            let items: &Vec<FeedItem> = match ctx.state.selected_source {
                FeedSource::Feed(feed_id) => {
                    if let Some(collection) = ctx.state.feed_items.get(&feed_id) {
                        &collection.items
                    } else {
                        &Vec::new()
                    }
                }
                FeedSource::Favourites => {
                    let mut favourites: Vec<FeedItem> = vec![];
                    for collection in ctx.state.feed_items.values() {
                        for item in &collection.items {
                            if ctx.state.favourites.contains(&item.id) {
                                favourites.push(item.clone());
                            }
                        }
                    }
                    &favourites.clone()
                }
                FeedSource::Readlist => {
                    let mut readlist: Vec<FeedItem> = vec![];
                    for collection in ctx.state.feed_items.values() {
                        for item in &collection.items {
                            if ctx.state.readlist.contains(&item.id) {
                                readlist.push(item.clone());
                            }
                        }
                    }
                    &readlist.clone()
                }
            };

            let feed_list_view = FeedListView {
                focused: false,
                title: match ctx.state.selected_source {
                    FeedSource::Feed(_) => {
                        let feed = &ctx.state.feeds[ctx.state.selected_feed_index];
                        &feed.title.as_str()
                    }
                    FeedSource::Favourites => "Favourites",
                    FeedSource::Readlist => "Readlist",
                },
                subtitle: match ctx.state.selected_source {
                    FeedSource::Feed(_) => {
                        let feed = &ctx.state.feeds[ctx.state.selected_feed_index];
                        &feed.subtitle.as_str()
                    }
                    FeedSource::Favourites => "All your favourite feeds",
                    FeedSource::Readlist => "All your articles from readlist",
                },
                // current_feed: Some(feed.clone()),
                feed_items: items,
                selected_feed_item_index: self.state.selected_feed_item_index,
                theme: ctx.config.current_theme,
                // subtitle: todo!(),
            };

            if self.show_sidebar {
                let [sidebar_area, feed_area] = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(25), Constraint::Fill(1)])
                    .areas(content_area);

                let sidebar_view = SidebarView {
                    items: self.state.filtered_feeds.clone(),
                    selected_feed_index: ctx.state.selected_feed_index,
                    selected_navigation_index: match ctx.state.selected_source {
                        FeedSource::Feed(_) => 0,
                        FeedSource::Favourites => 1,
                        FeedSource::Readlist => 2,
                    },
                    feed_count: ctx.state.feeds.len(),
                    favourite_count: ctx.state.favourites.len(),
                    readlist_count: ctx.state.readlist.len(),
                    theme: ctx.config.current_theme,
                };
                let sidebar = Sidebar::new(&sidebar_view);
                sidebar.render_ref(sidebar_area, frame.buffer_mut());

                let feed_list = FeedList::new(&feed_list_view);
                feed_list.render_ref(feed_area, frame.buffer_mut());
            } else {
                let feed_list = FeedList::new(&feed_list_view);
                feed_list.render_ref(content_area, frame.buffer_mut());
            }
        }

        if self.state.focused_widget == FocusedWidget::AddFeed {
            let add_feed = AddFeed::new(
                self.state.new_feed_title.clone(),
                self.state.new_feed_url.clone(),
                self.state.add_feed_index,
                ctx.config.current_theme,
            );
            add_feed.render_ref(inner, frame.buffer_mut());
        }
    }

    fn handle_input(&mut self, event: &Event, ctx: &ScreenContextMut) {
        self.filter_feeds(&ctx.state.feeds, &ctx.state.selected_source);
        match event {
            Event::Tick => {}
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    if self.state.focused_widget == FocusedWidget::Search {
                        match key.code {
                            KeyCode::Char('/') => {
                                self.state.focused_widget = FocusedWidget::FeedList;
                            }
                            KeyCode::Backspace => {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    self.state.search_query.clear();
                                } else {
                                    self.state.search_query.pop();
                                }
                            }
                            KeyCode::Char(c) => {
                                self.state.search_query.push(c);
                            }
                            _ => {}
                        }
                        return;
                    } else if self.state.focused_widget == FocusedWidget::AddFeed {
                        match key.code {
                            KeyCode::Esc => {
                                if self.state.focused_widget == FocusedWidget::AddFeed {
                                    self.state.add_feed_index = 0;
                                    self.state.new_feed_title = String::new();
                                    self.state.new_feed_url = String::new();
                                    self.state.focused_widget = FocusedWidget::FeedList;
                                    let _ = ctx.command_tx.send(AppCommand::CloseAddFeedPopup);
                                }
                            }
                            KeyCode::Enter => {
                                if self.state.add_feed_index == 2 {
                                    self.add_feed(ctx.event_tx.clone());
                                    self.state.add_feed_index = 0;
                                    self.state.new_feed_title = String::new();
                                    self.state.new_feed_url = String::new();
                                    self.state.focused_widget = FocusedWidget::FeedList;
                                    let _ = ctx.command_tx.send(AppCommand::CloseAddFeedPopup);
                                    let _ = ctx
                                        .command_tx
                                        .send(AppCommand::SelectSource(FeedSource::Feed(0)));
                                }
                            }
                            KeyCode::Tab => {
                                self.state.add_feed_index += 1;
                                if self.state.add_feed_index > 2 {
                                    self.state.add_feed_index = 0;
                                }
                            }
                            KeyCode::BackTab => {
                                if self.state.add_feed_index == 0 {
                                    self.state.add_feed_index = 2;
                                } else {
                                    self.state.add_feed_index -= 1;
                                }
                            }
                            KeyCode::Char(ch) => match self.state.add_feed_index {
                                0 => self.state.new_feed_title.push(ch),
                                1 => self.state.new_feed_url.push(ch),
                                _ => {}
                            },
                            KeyCode::Backspace => match self.state.add_feed_index {
                                0 => {
                                    self.state.new_feed_title.pop();
                                }
                                1 => {
                                    self.state.new_feed_url.pop();
                                }
                                _ => {}
                            },
                            _ => {}
                        }
                        return;
                    }

                    match key.code {
                        KeyCode::Enter => match self.state.focused_widget {
                            FocusedWidget::FeedList => {
                                self.state.focused_widget = FocusedWidget::FeedItems;
                            }
                            FocusedWidget::FeedItems => {
                                if let Some(feed_item_index) = self.state.selected_feed_item_index {
                                    let _ =
                                        ctx.command_tx.send(AppCommand::OpenFeed(feed_item_index));
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Char('d') => {
                            if self.state.focused_widget == FocusedWidget::FeedList {
                                let _ = ctx.command_tx.send(AppCommand::DeleteSelectedFeed);
                            }
                        }
                        KeyCode::Char('r') => {
                            let _ = ctx.command_tx.send(AppCommand::StartBackgroundSync);
                        }
                        KeyCode::Char('a') => {
                            self.state.focused_widget = FocusedWidget::AddFeed;
                            let _ = ctx.command_tx.send(AppCommand::OpenAddFeedPopup);
                        }
                        KeyCode::Char('b') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.show_sidebar = !self.show_sidebar;
                                if self.show_sidebar {
                                    self.state.focused_widget = FocusedWidget::FeedList;
                                } else {
                                    self.state.focused_widget = FocusedWidget::FeedItems;
                                }
                            }
                        }
                        KeyCode::Left | KeyCode::Right => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                if self.state.focused_widget == FocusedWidget::FeedList {
                                    self.state.focused_widget = FocusedWidget::FeedItems
                                } else {
                                    self.state.focused_widget = FocusedWidget::FeedList
                                }
                            }
                        }
                        KeyCode::Char('h') => {
                            if self.state.focused_widget == FocusedWidget::FeedList {
                                self.state.focused_widget = FocusedWidget::FeedItems
                            } else {
                                self.state.focused_widget = FocusedWidget::FeedList
                            }
                        }
                        KeyCode::Char('l') => {
                            if self.state.focused_widget == FocusedWidget::FeedList {
                                self.state.focused_widget = FocusedWidget::FeedItems
                            } else {
                                self.state.focused_widget = FocusedWidget::FeedList
                            }
                        }
                        KeyCode::Char('n') => {
                            self.state.focused_widget =
                                if self.state.focused_widget == FocusedWidget::Categories {
                                    match ctx.state.selected_source {
                                        FeedSource::Feed(_) => FocusedWidget::FeedList,
                                        _ => FocusedWidget::FeedItems,
                                    }
                                } else {
                                    FocusedWidget::Categories
                                };
                        }
                        KeyCode::Down | KeyCode::Char('j') => match self.state.focused_widget {
                            FocusedWidget::FeedList => {
                                if self.state.filtered_feeds.is_empty() {
                                    return;
                                }
                                let feed_index = ctx.state.selected_feed_index.wrapping_add(1)
                                    % self.state.filtered_feeds.len();
                                let feed_id = self.state.filtered_feeds[feed_index].id;
                                let _ = ctx
                                    .command_tx
                                    .send(AppCommand::SelectSource(FeedSource::Feed(feed_id)));
                            }
                            FocusedWidget::FeedItems => {
                                self.state.selected_feed_item_index =
                                    match self.state.selected_feed_item_index {
                                        Some(index) => {
                                            let feed =
                                                &ctx.state.feeds[ctx.state.selected_feed_index];
                                            let collection = &ctx.state.feed_items[&feed.id];
                                            let item_count = collection.items.len();
                                            if item_count == 0 {
                                                Some(0)
                                            } else {
                                                Some(index.wrapping_add(1) % item_count)
                                            }
                                        }
                                        _ => Some(0),
                                    };
                            }
                            FocusedWidget::Categories => {
                                let next_source = match ctx.state.selected_source {
                                    FeedSource::Feed(_) => FeedSource::Favourites,
                                    FeedSource::Favourites => FeedSource::Readlist,
                                    FeedSource::Readlist => {
                                        FeedSource::Feed(ctx.state.feeds.first().unwrap().id)
                                    }
                                };
                                let _ = ctx.command_tx.send(AppCommand::SelectSource(next_source));
                                self.filter_feeds(&ctx.state.feeds, &ctx.state.selected_source);
                            }
                            FocusedWidget::Search => {}
                            FocusedWidget::AddFeed => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match self.state.focused_widget {
                            FocusedWidget::Categories => {
                                let next_source = match ctx.state.selected_source {
                                    FeedSource::Feed(_) => FeedSource::Readlist,
                                    FeedSource::Favourites => {
                                        FeedSource::Feed(ctx.state.feeds.first().unwrap().id)
                                    }
                                    FeedSource::Readlist => FeedSource::Favourites,
                                };
                                let _ = ctx.command_tx.send(AppCommand::SelectSource(next_source));
                                self.filter_feeds(&ctx.state.feeds, &ctx.state.selected_source);
                            }
                            FocusedWidget::FeedList => {
                                let mut feed_index = ctx.state.selected_feed_index;
                                if self.state.filtered_feeds.is_empty() {
                                    return;
                                }
                                if feed_index == 0 {
                                    feed_index = self.state.filtered_feeds.len() - 1;
                                } else {
                                    feed_index -= 1;
                                }
                                let feed_id = self.state.filtered_feeds[feed_index].id;
                                let _ = ctx
                                    .command_tx
                                    .send(AppCommand::SelectSource(FeedSource::Feed(feed_id)));
                            }
                            FocusedWidget::FeedItems => {
                                let feed = &ctx.state.feeds[ctx.state.selected_feed_index];
                                let collection = &ctx.state.feed_items[&feed.id];
                                let item_count = collection.items.len();
                                if item_count == 0 {
                                    self.state.selected_feed_item_index = Some(0);
                                    return;
                                }
                                self.state.selected_feed_item_index =
                                    match self.state.selected_feed_item_index {
                                        Some(index) => {
                                            if index == 0 {
                                                Some(item_count - 1)
                                            } else {
                                                Some(index - 1)
                                            }
                                        }
                                        _ => Some(item_count - 1),
                                    };
                            }
                            FocusedWidget::Search => {}
                            FocusedWidget::AddFeed => {}
                        },
                        KeyCode::Char('/') => {
                            self.state.focused_widget =
                                if self.state.focused_widget == FocusedWidget::Search {
                                    FocusedWidget::FeedList
                                } else {
                                    FocusedWidget::Search
                                };
                        }
                        _ => {}
                    };
                }
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {} // TODO: Set sidebar = false if the screen is too small
        }
    }

    fn reset(&mut self) {}
}
