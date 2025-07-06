use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{BorderType, List, ListDirection, ListState, Paragraph, Widget, WidgetRef};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget},
};

use crate::event::{Event, EventHandler};
use crate::models::feed::Feed;
use crate::screens::home::HomeState;
use crate::widgets::{Focusable, WidgetExt};

pub struct Sidebar {
    focused: bool,
    filtered_feeds: Vec<Feed>,
    search_query: String,
    selected_feed_index: usize,
}

impl Sidebar {
    pub fn new(feeds: Vec<Feed>) -> Self {
        let filtered_feeds = feeds.clone();
        Sidebar {
            filtered_feeds,
            focused: false,
            search_query: String::new(),
            selected_feed_index: 0,
        }
    }

    pub fn set_selected_feed(&mut self, selected_feed_index: usize) {
        self.selected_feed_index = selected_feed_index;
    }

    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        let input = Paragraph::new(self.search_query.as_str())
            .style(Style::default())
            .block(Block::bordered().title("Search"));
        input.render(area, buf);
    }

    fn render_feed_list(&self, area: Rect, buf: &mut Buffer) {
        let list = List::new(self.filtered_feeds.iter().map(|item| item.title.clone()))
            .block(Block::bordered().border_type(BorderType::Plain))
            .style(Style::new().white())
            .highlight_style(Style::new().bold().bg(Color::Cyan))
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let mut state = ListState::default().with_selected(Some(self.selected_feed_index));
        StatefulWidget::render(list, area, buf, &mut state);
    }
}

impl WidgetRef for Sidebar {
    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let container = Block::default()
            // .title("  Feeds ")
            .style(match self.focused {
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
}

impl WidgetExt<HomeState> for Sidebar {
    fn update(&mut self, state: HomeState) {
        self.selected_feed_index = state.selected_feed_index;
    }

    fn handle_input(&mut self, event: &crate::event::Event) {
        match event {
            Event::Tick => {}
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => self.search_query.push(c),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
