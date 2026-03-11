use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{
        Block, Borders, List, ListDirection, ListState, Padding, Paragraph, StatefulWidget, Widget,
        WidgetRef, Wrap,
    },
};

use crate::models::{feed::Feed, feed_item::FeedItem, theme::Theme};

pub struct FeedListView<'a> {
    pub focused: bool,
    // pub current_feed: Option<Feed>,
    pub title: &'a str,
    pub subtitle: &'a str,
    pub feed_items: &'a Vec<FeedItem>,
    pub selected_feed_item_index: Option<usize>,
    pub theme: Theme,
}

pub struct FeedList<'a> {
    view: &'a FeedListView<'a>,
}

impl<'a> FeedList<'a> {
    pub fn new(view: &'a FeedListView) -> Self {
        Self { view }
    }
}

impl<'a> WidgetRef for FeedList<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = self.view.theme;
        let container = Block::default();

        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(4), Constraint::Fill(1)])
            .areas(container.inner(area));

        container.render(area, buf);

        let [title_area, subtitle_area] = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Fill(1), Constraint::Fill(2)])
            .areas(header_area);

        let title = Paragraph::new(self.view.title).fg(self.view.theme.text);
        let subtitle = Paragraph::new(self.view.subtitle)
            .wrap(Wrap { trim: true })
            .fg(self.view.theme.text);

        title.render(title_area, buf);
        subtitle.render(subtitle_area, buf);

        if self.view.feed_items.is_empty() {
            let empty_message =
                Paragraph::new("No feed items available").block(Block::new().title("Empty"));
            empty_message.render(content_area, buf);
        } else {
            let list = List::new(self.view.feed_items.iter().map(|item| {
                let mut text = Text::default();
                text.push_line("");
                text.push_line(Line::from(format!("  {} ", item.title.clone()).bold()));
                // TODO: Add author and other info
                text.push_line(Line::from(format!("   {} ", item.summary.clone())));
                text.push_line("");
                text
            }))
            .block(
                Block::default()
                    .padding(Padding::symmetric(1, 1))
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(theme.border)),
            )
            .style(Style::new().fg(theme.text))
            .highlight_style(Style::new().bold().bg(theme.primary).fg(Color::White))
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

            let mut state = ListState::default().with_selected(self.view.selected_feed_item_index);
            StatefulWidget::render(list, content_area, buf, &mut state);
        }
    }
}
