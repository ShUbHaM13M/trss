use ratatui::prelude::Constraint;
use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders},
};

use std::collections::HashMap;

use crate::app::AppEvent;
use crate::models::db::Database;
use crate::models::feed::Feed;
use crate::models::feed_item::FeedItem;
use crate::widgets::WidgetExt;
use crate::widgets::feed_list::FeedList;
use crate::{event::Event, screens::Screen, widgets::sidebar::Sidebar};

#[derive(Clone)]
pub struct HomeState {
    pub selected_feed_index: usize,
    pub selected_feed_item_index: Option<usize>,
    pub focused_element: String,
    pub current_feed_id: Option<i32>,
    pub feed_items: HashMap<i32, Vec<FeedItem>>,
    pub filtered_feeds: Vec<Feed>,
}

pub struct Home {
    widgets: HashMap<String, Box<dyn WidgetExt<HomeState>>>,
    show_sidebar: bool,

    feeds: Vec<Feed>,
    state: HomeState,
    database: Database,
}

impl Home {
    pub async fn new(database: Database) -> Self {
        let mut feeds: Vec<Feed> = Vec::new();
        if let Ok(f) = Feed::get_all(&database).await {
            feeds = f;
            // feeds = f[0..2].to_vec();
        }
        let mut feed_items: HashMap<i32, Vec<FeedItem>> =
            feeds.iter().map(|feed| return (feed.id, vec![])).collect();

        for feed in &feeds {
            let feed_id = feed.id;
            match FeedItem::get_by_feed_id(feed_id, &database).await {
                Ok(feed_item) => feed_items.insert(feed_id, feed_item),
                Err(e) => panic!("Failed to fetch feed items for feed id {}", e),
            };
        }

        let sidebar = Sidebar::new(feeds.clone());
        let selected_feed = feeds.get(0).unwrap().clone();
        let feed_list = FeedList::new(
            selected_feed.clone(),
            feed_items.get(&selected_feed.id).unwrap().clone(),
        );
        let mut widgets: HashMap<String, Box<dyn WidgetExt<HomeState>>> = HashMap::new();
        widgets.insert(String::from("sidebar"), Box::new(sidebar));
        widgets.insert(String::from("feed_list"), Box::new(feed_list));

        let state = HomeState {
            selected_feed_index: 0,
            selected_feed_item_index: None,
            focused_element: String::from("sidebar"),
            current_feed_id: None,
            feed_items,
            filtered_feeds: feeds.clone(),
        };

        Home {
            widgets,
            feeds,
            show_sidebar: true,
            state,
            database,
        }
    }

    fn update(&mut self) {
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
        app: &crate::app::App,
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

    fn handle_input(&mut self, event: &crate::event::EventHandler) -> Option<AppEvent> {
        let event = event.next().unwrap();
        if let Some(widget) = self.widgets.get_mut(&self.state.focused_element) {
            widget.handle_input(&event);
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
                            let widgets: Vec<&str> =
                                self.widgets.keys().map(|k| k.as_str()).collect();
                            let mut focused_element_index: i32 = widgets
                                .iter()
                                .position(|&k| k == self.state.focused_element.as_str())
                                .unwrap_or(0)
                                as i32;
                            focused_element_index -= 1;
                            if focused_element_index < 0 {
                                focused_element_index = (widgets.len() as i32) - 1;
                            }
                            match widgets.get(focused_element_index as usize) {
                                Some(widget) => {
                                    self.state.focused_element = widget.to_string();
                                }
                                _ => {
                                    self.state.focused_element = widgets[0].to_string();
                                }
                            }
                            return None;
                        }
                        KeyCode::Tab => {
                            let widgets: Vec<&str> =
                                self.widgets.keys().map(|k| k.as_str()).collect();
                            let mut focused_element_index = widgets
                                .iter()
                                .position(|&k| k == self.state.focused_element.as_str())
                                .unwrap_or(0);
                            focused_element_index += 1;
                            if focused_element_index > widgets.len() {
                                focused_element_index = 0;
                            }
                            match widgets.get(focused_element_index as usize) {
                                Some(widget) => {
                                    self.state.focused_element = widget.to_string();
                                }
                                _ => {
                                    self.state.focused_element = widgets[0].to_string();
                                }
                            }
                            return None;
                        }
                        KeyCode::Down => {
                            if self.show_sidebar && self.state.focused_element.as_str() == "sidebar"
                            {
                                self.state.selected_feed_index += 1;
                                if self.state.selected_feed_index >= self.state.filtered_feeds.len()
                                {
                                    self.state.selected_feed_index = 0;
                                }
                            }
                            return None;
                        }
                        KeyCode::Up => {
                            if self.show_sidebar && self.state.focused_element.as_str() == "sidebar"
                            {
                                self.state.selected_feed_index =
                                    self.state.selected_feed_index.wrapping_sub(1);
                                if self.state.selected_feed_index >= self.state.filtered_feeds.len()
                                {
                                    self.state.selected_feed_index =
                                        self.state.filtered_feeds.len() - 1;
                                }
                            }
                            return None;
                        }
                        // KeyCode::Char(c) => {
                        //     if self.show_sidebar && self.state.focused_element.as_str() == "sidebar"
                        //     {
                        //         self.state.search_query.push(c);
                        //     }
                        //     return None;
                        // }
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
}
