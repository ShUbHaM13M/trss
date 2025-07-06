use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        Block, Borders, List, ListDirection, ListState, Paragraph, StatefulWidget, Widget,
        WidgetRef,
    },
};

use crate::{
    models::{feed::Feed, feed_item::FeedItem},
    screens::home::HomeState,
    widgets::{Focusable, WidgetExt},
};

pub struct FeedList {
    pub focused: bool,
    pub current_feed: Feed,
    pub feed_items: Vec<FeedItem>,
}

impl FeedList {
    pub fn new(current_feed: Feed, feed_items: Vec<FeedItem>) -> Self {
        Self {
            focused: false,
            current_feed,
            feed_items,
        }
    }
    pub fn update_feed(&mut self, current_feed: Feed, current_feed_items: Vec<FeedItem>) {
        self.current_feed = current_feed;
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
}

impl WidgetRef for FeedList {
    // type State = ();

    fn render_ref(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let container = Block::default().style(match self.focused {
            true => Style::default().fg(Color::White).bg(Color::Rgb(30, 30, 46)),
            false => Style::default().fg(Color::White).bg(Color::Black),
        });

        container.render(area, buf);
        let inner_area = Rect::new(area.x + 2, area.y + 1, area.width - 5, area.height - 3);
        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(10), Constraint::Fill(1)])
            .margin(1)
            .areas(inner_area);

        // let container = Block::default()
        //     .title(Line::from(self.current_feed.title).left_aligned())
        //     .style(Style::default());

        // container.inner(area);

        let header = Paragraph::new(self.current_feed.subtitle.clone()).block(Block::new().title(
            format!("{} - {}", self.current_feed.title, self.feed_items.len()),
        ));

        header.render(header_area, buf);

        if self.feed_items.is_empty() {
            let empty_message =
                Paragraph::new("No feed items available").block(Block::new().title("Empty"));
            empty_message.render(content_area, buf);
        } else {
            let list = List::new(self.feed_items.iter().map(|item| item.title.clone()))
                .block(Block::bordered().borders(Borders::TOP))
                .style(Style::new().white())
                .highlight_style(Style::new().bold().on_cyan())
                // .highlight_symbol("> ")
                // .highlight_spacing(HighlightSpacing::Always)
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            StatefulWidget::render(list, content_area, buf, &mut ListState::default());
        }

        // let list = List::new(self.feed_items.iter().map(|item| item.title.clone()))
        //     .block(Block::bordered().borders(Borders::TOP))
        //     .style(Style::new().white())
        //     .highlight_style(Style::new().bold().on_cyan())
        //     // .highlight_symbol("> ")
        //     // .highlight_spacing(HighlightSpacing::Always)
        //     .repeat_highlight_symbol(true)
        //     .direction(ListDirection::TopToBottom);

        // .border_style(Style::default().fg(Color::White))
        // .border_type(BorderType::Thick)
        // .borders(Borders::ALL);
    }
}

impl WidgetExt<HomeState> for FeedList {
    fn update(&mut self, state: HomeState) {
        let current_feed = state
            .filtered_feeds
            .get(state.selected_feed_index)
            .unwrap()
            .clone();
        let current_feed_items = state.feed_items.get(&current_feed.id).unwrap().clone();
        self.current_feed = current_feed;
        self.feed_items = current_feed_items;
    }

    fn handle_input(&mut self, event: &crate::event::Event) {}
}
