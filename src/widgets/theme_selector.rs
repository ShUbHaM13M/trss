use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Span, Text},
    widgets::{
        Block, Clear, List, ListDirection, ListState, Padding, StatefulWidget, Widget, WidgetRef,
    },
};

use crate::{models::theme::Theme, utils::centered};

pub struct ThemeSelector<'a> {
    themes: &'a Vec<String>,
    current_theme: Theme,
    selected_theme_index: Option<usize>,
}

impl<'a> ThemeSelector<'a> {
    pub fn new(
        themes: &'a Vec<String>,
        current_theme: Theme,
        selected_theme_index: Option<usize>,
    ) -> Self {
        Self {
            themes,
            current_theme,
            selected_theme_index,
        }
    }
}

impl<'a> WidgetRef for ThemeSelector<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = self.current_theme;
        let block = Block::bordered()
            .title(Span::from(" Select Theme ").fg(theme.text))
            .border_style(Style::default().fg(theme.border))
            .bg(theme.background);
        let area = centered(45, 40, area);
        Clear.render(area, buf);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let list = List::new(self.themes.iter().map(|name| {
            let text = Text::from(format!(" {}", name));
            text
        }))
        .block(Block::default().padding(Padding::symmetric(1, 1)))
        .style(Style::new().fg(theme.text))
        .highlight_style(Style::new().bold().bg(theme.primary).fg(theme.text))
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);
        let mut state = ListState::default().with_selected(self.selected_theme_index);
        StatefulWidget::render(list, inner_area, buf, &mut state);
    }
}
