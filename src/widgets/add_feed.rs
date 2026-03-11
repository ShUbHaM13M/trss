use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Clear, Paragraph, Widget, WidgetRef},
};

use crate::{models::theme::Theme, utils::centered};

pub struct AddFeed {
    pub feed_title_text: String,
    pub feed_url_text: String,
    pub current_theme: Theme,
    pub focused_index: usize,
}

impl AddFeed {
    pub fn new(
        feed_title_text: String,
        feed_url_text: String,
        focused_index: usize,
        current_theme: Theme,
    ) -> Self {
        Self {
            feed_title_text,
            feed_url_text,
            current_theme,
            focused_index,
        }
    }

    fn render_title_input(&self, area: Rect, buf: &mut Buffer) {
        let focused = self.focused_index == 0;

        let mut text = self.feed_title_text.as_str();
        let mut style = Style::default();

        let mut block = Block::bordered()
            .title(Span::from(" Feed title ").fg(if focused {
                self.current_theme.primary
            } else {
                self.current_theme.text
            }))
            .fg(self.current_theme.text)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.current_theme.border));
        if focused {
            block = block.border_style(Style::default().fg(self.current_theme.primary));
        }
        if text.is_empty() {
            text = "Enter feed title";
            // TODO: Add DIM text colour
            style = style.fg(Color::DarkGray);
        }
        let input = Paragraph::new(format!(" {}", text))
            .style(style)
            .block(block);
        input.render(area, buf);
    }

    fn render_url_input(&self, area: Rect, buf: &mut Buffer) {
        let focused = self.focused_index == 1;

        let mut text = self.feed_url_text.as_str();
        let mut style = Style::default();
        let mut block = Block::bordered()
            .title(Span::from(" Feed URL ").fg(if focused {
                self.current_theme.primary
            } else {
                self.current_theme.text
            }))
            .fg(self.current_theme.text)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.current_theme.border));
        if focused {
            block = block
                // .border_type(BorderType::Thick)
                .border_style(Style::default().fg(self.current_theme.primary));
        }
        if text.is_empty() {
            text = "Enter feed url";
            // TODO: Add DIM text colour
            style = style.fg(Color::DarkGray);
        }
        let input = Paragraph::new(format!(" {}", text))
            .style(style)
            .block(block);
        input.render(area, buf);
    }

    fn render_add_button(&self, area: Rect, buf: &mut Buffer) {
        let focused = self.focused_index == 2;

        let mut block = Block::bordered()
            .fg(Color::White)
            .border_style(
                Style::default()
                    .fg(self.current_theme.border)
                    .bg(self.current_theme.background),
            )
            .border_type(BorderType::Rounded);
        if focused {
            block = block.border_style(Style::default().fg(self.current_theme.primary));
        }
        // if focused_element == WIDGET_FEED_ADD {
        //     style = style.fg(Color::Blue);
        // }
        let add_button = Paragraph::new("Add New Feed")
            .block(block)
            .fg(if focused {
                self.current_theme.primary
            } else {
                self.current_theme.text
            })
            .centered();
        add_button.render(area, buf);
    }
}

impl WidgetRef for AddFeed {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let theme = self.current_theme;
        let block = Block::bordered()
            .title(Span::from(" Add New Feed ").fg(theme.text))
            .border_style(Style::default().fg(theme.border))
            .bg(theme.background);
        let area = centered(45, 45, area);
        Clear.render(area, buf);

        let inner_area = block.inner(area);
        let [add_title_area, add_url_area, add_button_area, _] = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .areas(inner_area);

        block.render(area, buf);
        self.render_title_input(add_title_area, buf);
        self.render_url_input(add_url_area, buf);
        self.render_add_button(add_button_area, buf);
    }
}

// impl WidgetExt<HomeState> for AddFeed {
//     fn update(&mut self, state: HomeState) {}

//     fn handle_input(
//         &mut self,
//         state: &HomeState,
//         event: &crate::event::Event,
//     ) -> Option<HomeState> {
//         let mut new_state = state.clone();
//         let focused_element = self.get_focused_element();
//         match event {
//             Event::Key(key) => {
//                 if key.kind == KeyEventKind::Press {
//                     match key.code {
//                         KeyCode::Char(c) => {
//                             if focused_element == WIDGET_FEED_TITLE {
//                                 self.feed_title_text.push(c);
//                             } else if focused_element == WIDGET_FEED_URL {
//                                 self.feed_url_text.push(c);
//                             }
//                             return None;
//                         }
//                         KeyCode::Backspace => {
//                             if key.modifiers.contains(KeyModifiers::CONTROL) {
//                                 if focused_element == WIDGET_FEED_TITLE {
//                                     self.feed_title_text.clear();
//                                 } else if focused_element == WIDGET_FEED_URL {
//                                     self.feed_url_text.clear();
//                                 }
//                             } else {
//                                 if focused_element == WIDGET_FEED_TITLE {
//                                     self.feed_title_text.pop();
//                                 } else if focused_element == WIDGET_FEED_URL {
//                                     self.feed_url_text.pop();
//                                 }
//                             }
//                             return None;
//                         }
//                         KeyCode::Tab => {
//                             if let Some(child_index) = self.get_child_focused_index() {
//                                 if child_index < self.widgets.len() - 1 {
//                                     self.set_child_focused_index(Some(child_index + 1));
//                                 } else {
//                                     self.set_child_focused_index(Some(0));
//                                 }
//                             } else {
//                                 self.set_child_focused_index(Some(0));
//                             }
//                         }
//                         KeyCode::Enter => {
//                             if focused_element == WIDGET_FEED_ADD {
//                                 println!("{}, {}", self.feed_title_text, self.feed_url_text);
//                             }
//                         }
//                         // KeyCode::Esc => {
//                         //     new_state.show_add_feed = false;
//                         //     return Some(new_state);
//                         // }
//                         _ => {}
//                     }
//                 }
//             }
//             _ => {}
//         }
//         None
//     }
//     fn get_child_widgets(&self) -> Option<Vec<String>> {
//         return Some(self.widgets.clone());
//     }
// }
