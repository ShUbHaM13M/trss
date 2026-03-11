use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, WidgetRef},
};

use crate::{app::Screens, models::theme::Theme};

pub struct Header<'a> {
    pub current_screen: Screens,
    pub theme: Theme,
    pub search_query: &'a str,
    pub search_focused: bool,
    pub syncing: bool,
}

impl<'a> WidgetRef for Header<'a> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let header = Block::default()
            .bg(self.theme.background)
            .fg(self.theme.text)
            .padding(Padding {
                left: 2,
                right: 2,
                top: 1,
                bottom: 0,
            })
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(self.theme.border));

        let [title_area, controls_area] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::SpaceBetween)
            .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
            .areas(header.inner(area));

        let title = Paragraph::new(Text::from(" Trss "))
            .left_aligned()
            .style(Style::default().fg(self.theme.text));

        header.render_ref(area, buf);
        title.render_ref(title_area, buf);

        let [search_area, sync_area] = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::End)
            .spacing(1)
            .constraints(vec![Constraint::Percentage(40), Constraint::Max(4)])
            .areas(controls_area);

        if self.current_screen == Screens::Home {
            let mut style = Style::default()
                .fg(self.theme.text)
                .bg(self.theme.background);

            if self.search_focused {
                style = style.bg(self.theme.primary).fg(self.theme.text).bold()
            }

            let search = Paragraph::new(Line::from(format!(
                " {} ",
                if self.search_query.is_empty() {
                    " Press '/' to Search "
                } else {
                    self.search_query
                }
            )))
            .style(style);

            // let block = Block::bordered().title(" Search ");
            // .borders(Borders::LEFT | Borders::RIGHT)
            // .border_style(Style::default());

            // let search = Paragraph::new(format!(" {}", self.search_query))
            //     .block(block)
            //     .style(Style::default().fg(self.theme.text));

            search.render_ref(search_area, buf);
        }

        if self.syncing {
            let sync =
                Paragraph::new(Span::from("  ").style(Style::default().fg(self.theme.primary)));
            sync.render_ref(sync_area, buf);
        }
    }
}
