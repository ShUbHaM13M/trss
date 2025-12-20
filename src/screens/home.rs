use ratatui::crossterm::event::KeyModifiers;
use ratatui::prelude::Constraint;
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders},
};

use std::collections::HashMap;
use std::sync::mpsc::{self, channel};

use crate::app::{AppEvent, AppState, Screens};
use crate::event::EventHandler;
use crate::models::db::Database;
use crate::models::feed::{Feed, spawn_update_feeds};
use crate::models::feed_item::FeedItem;
use crate::widgets::WidgetExt;
use crate::widgets::feed_list::FeedList;
use crate::{event::Event, screens::Screen, widgets::sidebar::Sidebar};

const SIDEBAR: &str = "sidebar";
const FEED_LIST: &str = "feed_list";

#[derive(Clone)]
pub struct HomeState {
    pub selected_feed_index: usize,
    pub selected_feed_item_index: Option<usize>,
    pub focused_element: String,
    pub feed_items: HashMap<i32, Vec<FeedItem>>,
    pub filtered_feeds: Vec<Feed>,
    pub feeds: Vec<Feed>,
}

pub struct Home {
    widgets: HashMap<String, Box<dyn WidgetExt<HomeState>>>,
    show_sidebar: bool,
    update_feeds_receiver: mpsc::Receiver<Vec<i32>>,

    state: HomeState,
    database: Database,
}

impl Home {
    pub async fn new(database: Database) -> Self {
        let mut feeds: Vec<Feed> = Vec::new();
        if let Ok(f) = Feed::get_all(&database).await {
            feeds = f;
        }
        // TODO: Adding configuration for background sync
        let background_sync = false;
        let (sender, receiver) = channel();
        if background_sync {
            spawn_update_feeds(feeds.clone(), sender).await;
        }
        let mut feed_items: HashMap<i32, Vec<FeedItem>> =
            feeds.iter().map(|feed| return (feed.id, vec![])).collect();

        for feed in &mut feeds {
            let feed_id = feed.id;
            match FeedItem::get_by_feed_id(feed_id, &database).await {
                Ok(feed_item) => {
                    feed.feed_count = feed_item.len() as i32;
                    feed_items.insert(feed_id, feed_item);
                }
                Err(e) => panic!("Failed to fetch feed items for feed id {}", e),
            };
        }

        let sidebar = Sidebar::new(feeds.clone());
        let selected_feed = feeds.get(0).or(None);
        let selected_feed_items = selected_feed
            .as_ref()
            .and_then(|f| feed_items.get(&f.id).cloned())
            .unwrap_or_default();
        let feed_list = FeedList::new(selected_feed.cloned(), selected_feed_items);
        let mut widgets: HashMap<String, Box<dyn WidgetExt<HomeState>>> = HashMap::new();
        widgets.insert(String::from(SIDEBAR), Box::new(sidebar));
        widgets.insert(String::from(FEED_LIST), Box::new(feed_list));

        let state = HomeState {
            selected_feed_index: 0,
            selected_feed_item_index: None,
            focused_element: String::from(FEED_LIST),
            feed_items,
            filtered_feeds: feeds.clone(),
            feeds,
        };

        Home {
            widgets,
            show_sidebar: true,
            state,
            database,
            update_feeds_receiver: receiver,
        }
    }

    fn update(&mut self) {
        match self.update_feeds_receiver.try_recv() {
            Ok(_) => {
                // println!("Received {} results", results.len());
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {}
        }
        for (id, widget) in self.widgets.iter_mut() {
            if *id == self.state.focused_element {
                widget.focus();
            } else {
                widget.blur();
            }
            widget.update(self.state.clone())
        }
    }
}

impl Screen for Home {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        _app: &crate::app::App,
    ) {
        let b: Block = Block::default()
            .title(Line::from(" trss ").centered())
            .style(Style::default().fg(Color::White))
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Double)
            .borders(Borders::ALL);

        frame.render_widget(b, area);

        // TODO: Update with some constant values
        let block_area = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);

        if self.show_sidebar {
            let [sidebar_area, main_area] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(block_area);

            let sidebar = self.widgets.get("sidebar").unwrap();
            sidebar.render_ref(sidebar_area, frame.buffer_mut());

            let feed_list = self.widgets.get("feed_list").unwrap();
            feed_list.render_ref(main_area, frame.buffer_mut());
        } else {
            let [main_area] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Fill(1)])
                .areas(block_area);

            let feed_list = self.widgets.get("feed_list").unwrap();
            feed_list.render_ref(main_area, frame.buffer_mut());
        }
    }

    fn handle_input(&mut self, event: &EventHandler, state: &AppState) -> Option<AppEvent> {
        let mut new_state = state.clone();
        let event = event.next().unwrap();
        for (id, widget) in self.widgets.iter_mut() {
            if *id == self.state.focused_element {
                if let Some(new_state) = widget.handle_input(&self.state, &event) {
                    self.state = new_state;
                }
            }
        }
        match event {
            Event::Tick => {
                self.update();
                return None;
            }
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            return Some(AppEvent::Quit);
                        }
                        KeyCode::BackTab => {
                            let widgets: Vec<String> =
                                self.widgets.keys().map(|k| k.to_string()).collect();
                            let widget_count = widgets.len();
                            let mut focused_element_index = widgets
                                .iter()
                                .position(|k| k == self.state.focused_element.as_str())
                                .unwrap_or(0);
                            if let Some(widget) = self
                                .widgets
                                .get_mut(widgets[focused_element_index].as_str())
                            {
                                if let Some(child_widgets) = widget.get_child_widgets() {
                                    if let Some(child_index) = widget.get_child_focused_index() {
                                        if child_index < child_widgets.len() - 1 {
                                            widget.set_child_focused_index(Some(
                                                child_index.wrapping_sub(1) % child_widgets.len(),
                                            ));
                                        } else {
                                            // FIXME: This is causing problem when the widget gets back the focus
                                            widget.set_child_focused_index(None);
                                            focused_element_index = focused_element_index
                                                .wrapping_sub(1)
                                                % widget_count;
                                        }
                                    } else {
                                        widget.set_child_focused_index(Some(0));
                                    }
                                } else {
                                    focused_element_index =
                                        focused_element_index.wrapping_sub(1) % widget_count;
                                }
                            }

                            self.state.focused_element = widgets[focused_element_index].to_string();
                            return None;
                        }
                        KeyCode::Tab => {
                            let widgets: Vec<String> =
                                self.widgets.keys().map(|k| k.to_string()).collect();
                            let widget_count = widgets.len();
                            let mut focused_element_index = widgets
                                .iter()
                                .position(|k| k == self.state.focused_element.as_str())
                                .unwrap_or(0);
                            if let Some(widget) = self
                                .widgets
                                .get_mut(widgets[focused_element_index].as_str())
                            {
                                if let Some(child_widgets) = widget.get_child_widgets() {
                                    if let Some(child_index) = widget.get_child_focused_index() {
                                        if child_index < child_widgets.len() - 1 {
                                            widget.set_child_focused_index(Some(child_index + 1));
                                        } else {
                                            // FIXME: This is causing problem when the widget gets back the focus
                                            widget.set_child_focused_index(None);
                                            focused_element_index += 1;
                                        }
                                    } else {
                                        widget.set_child_focused_index(Some(0));
                                    }
                                } else {
                                    focused_element_index += 1;
                                }
                            }
                            if focused_element_index >= widget_count {
                                focused_element_index = 0;
                            }
                            self.state.focused_element = widgets[focused_element_index].to_string();
                            return None;
                        }
                        KeyCode::Char('s') => {
                            // FIXME: This also sends events down to the sidebar inserting characters in the search bar
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                self.show_sidebar = !self.show_sidebar;
                            }
                            return None;
                        }
                        KeyCode::Enter => {
                            if self.state.focused_element == FEED_LIST {
                                if let Some(feed_item_index) = self.state.selected_feed_item_index {
                                    let feeds = self
                                        .state
                                        .filtered_feeds
                                        .get(self.state.selected_feed_index)
                                        .unwrap();

                                    let feed_items = self.state.feed_items.get(&feeds.id).unwrap();
                                    new_state.selected_feed_item =
                                        Some(feed_items.get(feed_item_index).unwrap().clone());
                                    return Some(AppEvent::ChangeScreen(
                                        Screens::ViewFeed,
                                        new_state,
                                    ));
                                    // return Some(AppEvent::ChangeScreen(Screens::ViewFeed));
                                }
                            }
                            return None;
                        }
                        _ => None::<AppEvent>,
                    };
                }
                None
            }
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
            // TODO: Set sidebar = false if the screen is too small
        }
    }

    fn reset(&mut self) {}
}
