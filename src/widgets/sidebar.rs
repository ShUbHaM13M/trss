use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{BorderType, List, ListDirection, ListState, Paragraph, Widget, WidgetRef};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, StatefulWidget},
};

use crate::event::Event;
use crate::models::feed::Feed;
use crate::screens::home::HomeState;
use crate::widgets::{Focusable, WidgetExt};

const WIDGET_SEARCH: &str = "Search";
const WIDGET_FEEDS: &str = "Feeds";

pub struct Sidebar {
    focused: bool,
    filtered_feeds: Vec<Feed>,
    search_query: String,
    selected_feed_index: usize,
    widgets: Vec<String>,
    focused_widget_index: Option<usize>,
}

impl Sidebar {
    pub fn new(feeds: Vec<Feed>) -> Self {
        let filtered_feeds = feeds.clone();
        Sidebar {
            filtered_feeds,
            focused: false,
            search_query: String::new(),
            selected_feed_index: 0,
            widgets: vec![String::from(WIDGET_SEARCH), String::from(WIDGET_FEEDS)],
            focused_widget_index: Some(0),
        }
    }

    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        let focused_element = self.get_focused_element();
        let search_focused = focused_element == WIDGET_SEARCH;

        let mut text = self.search_query.as_str();
        let mut style = Style::default();
        let mut block = Block::bordered().title(WIDGET_SEARCH).fg(Color::White);
        if search_focused {
            block = block.border_type(BorderType::Double);
        } else {
            block = block.border_type(BorderType::Plain);
        }
        if self.search_query.is_empty() {
            text = " Search (Press '/' to focus) ";
            // TODO: Add DIM text colour
            style = style.fg(Color::DarkGray);
        }
        let input = Paragraph::new(format!(" {}", text))
            .style(style)
            .block(block);
        input.render(area, buf);
    }

    fn render_feed_list(&self, area: Rect, buf: &mut Buffer) {
        let focused_element = self.get_focused_element();
        let feeds_focused = focused_element == WIDGET_FEEDS;
        let mut block = Block::bordered();
        if feeds_focused {
            block = block.border_type(BorderType::Double);
        } else {
            block = block.border_type(BorderType::Plain);
        }

        let feed_titles = self.filtered_feeds.iter().map(|item| {
            let feed_items_count = (item.feed_count.to_string().len() + 5) as u16;
            let title_width = item.title.len();
            let available_width: usize = area.width.saturating_sub(feed_items_count) as usize;
            let truncated_title = if title_width > available_width {
                format!("{}...", &item.title.as_str()[..available_width - 3])
            } else {
                item.title.to_string()
            };
            return Text::from(format!(
                " {:<width$} {} ",
                truncated_title,
                item.feed_count,
                width = available_width
            ));
        });

        let list = List::new(feed_titles)
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().bold().bg(Color::Cyan))
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let mut state = ListState::default().with_selected(Some(self.selected_feed_index));
        StatefulWidget::render(list, area, buf, &mut state);
    }

    fn get_focused_element(&self) -> &str {
        let mut focused_element = "";
        if let Some(index) = self.focused_widget_index {
            if let Some(element) = self.widgets.get(index) {
                focused_element = element;
            }
        }
        focused_element
    }

    fn filter_feeds(&mut self, feeds: &Vec<Feed>) {
        if self.search_query.is_empty() {
            self.filtered_feeds = feeds.clone();
            return;
        }
        self.filtered_feeds = feeds
            .iter()
            .filter(|feed| {
                feed.title
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
            })
            .cloned()
            .collect();
    }
}

impl WidgetRef for Sidebar {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let container = Block::default().style(match self.focused {
            true => Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 46)),
            false => Style::default().fg(Color::White).bg(Color::Black),
        });
        let inner_area = Rect::new(area.x + 2, area.y + 1, area.width - 5, area.height - 1);
        let [search_area, feed_list_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .areas(inner_area);
        container.render(area, buf);
        self.render_search(search_area, buf);
        self.render_feed_list(feed_list_area, buf);
    }
}

impl Focusable for Sidebar {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }

    fn get_child_focused_index(&self) -> Option<usize> {
        self.focused_widget_index
    }

    fn set_child_focused_index(&mut self, index: Option<usize>) {
        self.focused_widget_index = index;
    }
}

impl WidgetExt<HomeState> for Sidebar {
    fn update(&mut self, state: HomeState) {
        self.selected_feed_index = state.selected_feed_index;
    }

    fn handle_input(
        &mut self,
        state: &HomeState,
        event: &crate::event::Event,
    ) -> Option<HomeState> {
        let mut new_state = state.clone();
        let focused_element = self.get_focused_element();
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => {
                            if focused_element == WIDGET_SEARCH {
                                self.search_query.push(c);
                                self.filter_feeds(&state.feeds);
                                new_state.filtered_feeds = self.filtered_feeds.clone();
                            }
                            return Some(new_state);
                        }
                        KeyCode::Backspace => {
                            if focused_element == WIDGET_SEARCH {
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // TODO: This does not work in kitty
                                    self.search_query.clear();
                                } else {
                                    self.search_query.pop();
                                }
                                self.filter_feeds(&state.feeds);
                                new_state.filtered_feeds = self.filtered_feeds.clone();
                            }
                            return Some(new_state);
                        }
                        KeyCode::Down => {
                            if focused_element == WIDGET_FEEDS {
                                self.selected_feed_index = self.selected_feed_index.wrapping_add(1)
                                    % self.filtered_feeds.len();
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    self.selected_feed_index = self.filtered_feeds.len() - 1;
                                }
                                new_state.selected_feed_index = self.selected_feed_index;
                            }

                            return Some(new_state);
                        }
                        KeyCode::Up => {
                            if focused_element == WIDGET_FEEDS {
                                self.selected_feed_index = if self.selected_feed_index == 0 {
                                    self.filtered_feeds.len() - 1
                                } else {
                                    self.selected_feed_index - 1
                                };
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    self.selected_feed_index = 0;
                                }
                                new_state.selected_feed_index = self.selected_feed_index;
                            }
                            return Some(new_state);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        None
    }
    fn get_child_widgets(&self) -> Option<Vec<String>> {
        return Some(self.widgets.clone());
    }
}
