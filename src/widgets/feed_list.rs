use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        Block, Borders, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget,
        WidgetRef, Wrap,
    },
};

use crate::{
    event::Event,
    models::{feed::Feed, feed_item::FeedItem},
    screens::home::HomeState,
    widgets::{Focusable, WidgetExt},
};

pub struct FeedList {
    focused: bool,
    current_feed: Option<Feed>,
    feed_items: Vec<FeedItem>,
    selected_feed_item_index: Option<usize>,
}

impl FeedList {
    pub fn new(current_feed: Option<Feed>, feed_items: Vec<FeedItem>) -> Self {
        let cf = if current_feed.is_none() {
            None
        } else {
            current_feed.clone()
        };
        Self {
            focused: false,
            current_feed: cf,
            feed_items,
            selected_feed_item_index: None,
        }
    }
    pub fn update_feed(&mut self, current_feed: Feed, current_feed_items: Vec<FeedItem>) {
        self.current_feed = Some(current_feed);
        self.feed_items = current_feed_items;
    }
}

impl Focusable for FeedList {
    fn focus(&mut self) {
        self.focused = true;
    }

    fn blur(&mut self) {
        self.focused = false;
    }
    fn get_child_focused_index(&self) -> Option<usize> {
        None
    }
    fn set_child_focused_index(&mut self, _index: Option<usize>) {}
}

impl WidgetRef for FeedList {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let container = Block::default().style(match self.focused {
            true => Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 46)),
            false => Style::default().fg(Color::White).bg(Color::Black),
        });

        container.render(area, buf);
        let inner_area = Rect::new(area.x + 2, area.y + 1, area.width - 3, area.height - 2);
        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(10), Constraint::Fill(1)])
            .margin(1)
            .areas(inner_area);

        if let Some(current_feed) = self.current_feed.clone() {
            let header = Paragraph::new(current_feed.subtitle.as_str())
                .block(Block::new().title(current_feed.title.as_str()).bold())
                .wrap(Wrap { trim: true });
            header.render(header_area, buf);
        }

        if self.feed_items.is_empty() {
            let empty_message =
                Paragraph::new("No feed items available").block(Block::new().title("Empty"));
            empty_message.render(content_area, buf);
        } else {
            let list = List::new(self.feed_items.iter().map(|item| item.title.clone()))
                .block(Block::bordered().borders(Borders::TOP))
                .style(Style::new().white())
                .highlight_style(Style::new().bold().on_cyan())
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            let mut state = ListState::default().with_selected(self.selected_feed_item_index);
            StatefulWidget::render(list, content_area, buf, &mut state);
        }
    }
}

impl WidgetExt<HomeState> for FeedList {
    fn update(&mut self, state: HomeState) {
        if let Some(current_feed) = state.filtered_feeds.get(state.selected_feed_index) {
            self.current_feed = Some(current_feed.clone());
            if let Some(current_feed_items) = state.feed_items.get(&current_feed.id) {
                self.feed_items = current_feed_items.clone();
            }
        } else {
            self.current_feed = None;
            self.feed_items = Vec::new();
        }
    }

    fn handle_input(
        &mut self,
        state: &HomeState,
        event: &crate::event::Event,
    ) -> Option<HomeState> {
        let mut new_state = state.clone();
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Down => {
                            self.selected_feed_item_index = match self.selected_feed_item_index {
                                Some(index) => Some(index.wrapping_add(1) % self.feed_items.len()),
                                _ => Some(0),
                            };
                            new_state.selected_feed_item_index = self.selected_feed_item_index;
                            return Some(new_state);
                        }
                        KeyCode::Up => {
                            self.selected_feed_item_index = match self.selected_feed_item_index {
                                Some(index) => {
                                    if index == 0 {
                                        Some(self.feed_items.len() - 1)
                                    } else {
                                        Some(index - 1)
                                    }
                                }
                                _ => Some(self.feed_items.len() - 1),
                            };
                            new_state.selected_feed_item_index = self.selected_feed_item_index;
                            return Some(new_state);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        return None;
    }
    fn get_child_widgets(&self) -> Option<Vec<String>> {
        None
    }
}
