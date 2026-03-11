use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::Text;
use ratatui::widgets::{
    BorderType, Borders, List, ListDirection, ListState, Padding, Paragraph, Widget, WidgetRef,
};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, StatefulWidget},
};

use crate::models::feed::Feed;
use crate::models::theme::Theme;

pub struct NavigationItem {
    icon: &'static str,
    label: &'static str,
    count: usize,
}

pub struct SidebarView {
    pub items: Vec<Feed>,
    pub selected_feed_index: usize,
    pub selected_navigation_index: usize,
    pub feed_count: usize,
    pub favourite_count: usize,
    pub readlist_count: usize,
    pub theme: Theme,
}

pub struct Sidebar<'a> {
    view: &'a SidebarView,
    navigation_items: [NavigationItem; 3],
}

impl<'a> Sidebar<'a> {
    pub fn new(view: &'a SidebarView) -> Self {
        Self {
            view,
            navigation_items: [
                NavigationItem {
                    icon: "",
                    label: "All Articles",
                    count: view.feed_count,
                },
                NavigationItem {
                    icon: "",
                    label: "Favorites",
                    count: view.favourite_count,
                },
                NavigationItem {
                    icon: "",
                    label: "Reading List",
                    count: view.readlist_count,
                },
            ],
        }
    }

    fn render_navigation(&self, area: Rect, buf: &mut Buffer) {
        let theme = &self.view.theme;
        let container = Block::default()
            .fg(theme.text)
            .padding(Padding::top(1))
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.border));

        let items = Layout::default()
            .direction(Direction::Vertical)
            .constraints(self.navigation_items.iter().map(|_| Constraint::Fill(1)))
            .split(container.inner(area));

        container.render(area, buf);

        for (index, i) in self.navigation_items.iter().enumerate() {
            let mut item = Block::default()
                .padding(Padding::horizontal(1))
                .border_style(Style::default().fg(Color::White).bg(Color::Black))
                .border_type(BorderType::Rounded);
            if index == self.view.selected_navigation_index {
                item = item.bg(theme.primary).fg(Color::White);
            }
            let left = Paragraph::new(format!("{} {}", i.icon, i.label));
            let right = Paragraph::new(format!("[{}]", i.count)).right_aligned();

            let [left_area, right_area] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::SpaceBetween)
                .constraints([Constraint::Fill(1), Constraint::Fill(1)])
                .areas(item.inner(items[index]));

            item.render(items[index], buf);
            left.render(left_area, buf);
            right.render(right_area, buf);
        }
    }

    fn render_feed_list(&self, area: Rect, buf: &mut Buffer) {
        let theme = &self.view.theme;
        let block = Block::default()
            .fg(theme.text)
            .padding(Padding::vertical(1));

        let feed_titles = self.view.items.iter().map(|item| {
            let feed_items_count = (item.feed_count.to_string().len() + 5) as u16;
            let title_width = item.title.len();
            let available_width: usize = area.width.saturating_sub(feed_items_count) as usize;
            let truncated_title = if title_width > available_width {
                // FIX: on lower screen sizes causes panic
                // byte index 19 is not a char boundary; it is inside '–' (bytes 18..21) of `self driving cars – Ars Technica`
                format!("{}...", &item.title.as_str()[..available_width - 3])
            } else {
                item.title.to_string()
            };
            return Text::from(format!(
                " {:<width$} [{}] ",
                truncated_title,
                item.feed_count,
                width = available_width
            ));
        });

        let list = List::new(feed_titles)
            .block(block)
            .style(Style::default().fg(theme.text))
            .highlight_style(Style::new().bold().bg(theme.primary).fg(Color::White))
            .repeat_highlight_symbol(true)
            .direction(ListDirection::TopToBottom);

        let mut state = ListState::default().with_selected(Some(self.view.selected_feed_index));
        StatefulWidget::render(list, area, buf, &mut state);
    }
}

impl<'a> WidgetRef for Sidebar<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = self.view.theme;
        let container = Block::default()
            .padding(Padding::horizontal(1))
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(theme.border));

        let [navigation_area, feed_list_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(5), Constraint::Fill(1)])
            .areas(container.inner(area));

        container.render(area, buf);
        self.render_navigation(navigation_area, buf);
        self.render_feed_list(feed_list_area, buf);
    }
}
