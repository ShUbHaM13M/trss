use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::{Span, Text},
    widgets::{Block, Padding, Paragraph, Widget, WidgetRef},
};

use crate::{app::Screens, models::theme::Theme};

pub struct Footer<'a> {
    pub current_screen: Screens,
    pub theme: Theme,
    pub message: Option<&'a str>,
}

#[derive(Clone)]
struct KeyBinding {
    key: String,
    label: String,
}

lazy_static! {
    static ref SCREEN_TIPS_MAP: HashMap<Screens, Vec<KeyBinding>> = {
        let mut m = HashMap::new();
        m.insert(
            Screens::Home,
            vec![
                KeyBinding {
                    key: String::from("j/k"),
                    label: String::from("Move"),
                },
                KeyBinding {
                    key: String::from("h/l"),
                    label: String::from("Focus"),
                },
                KeyBinding {
                    key: String::from("gg/G"),
                    label: String::from("Top/End"),
                },
                KeyBinding {
                    key: String::from("Enter"),
                    label: String::from("Open"),
                },
                KeyBinding {
                    key: String::from("/"),
                    label: String::from("Search"),
                },
            ],
        );
        m.insert(
            Screens::ViewFeed,
            vec![
                KeyBinding {
                    key: String::from("Esc"),
                    label: String::from("Back"),
                },
                KeyBinding {
                    key: String::from("j/k"),
                    label: String::from("Scroll"),
                },
                KeyBinding {
                    key: String::from("PgUp/Dn"),
                    label: String::from("Page"),
                },
                KeyBinding {
                    key: String::from("g/G"),
                    label: String::from("Top/End"),
                },
                KeyBinding {
                    key: String::from("n/p"),
                    label: String::from("Next/Prev"),
                },
                KeyBinding {
                    key: String::from("f/r"),
                    label: String::from("Fav/Save"),
                },
            ],
        );
        m
    };
}

impl WidgetRef for Footer<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let container = Block::default()
            .padding(Padding::symmetric(2, 1))
            .bg(self.theme.primary)
            .fg(Color::White);

        let inner_area = container.inner(area);
        if let Some(message) = &self.message {
            container.render(area, buf);
            Paragraph::new(*message)
                .fg(Color::White)
                .bg(self.theme.primary)
                .render(inner_area, buf);
            return;
        }

        container.render(area, buf);
        let Some(tips) = SCREEN_TIPS_MAP.get(&self.current_screen) else {
            return;
        };

        let parts = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Start)
            .constraints(
                tips.iter()
                    .map(|a| Constraint::Length((a.key.len() + a.label.len() + 3) as u16)),
            )
            .spacing(2)
            .split(inner_area);

        for (index, part) in parts.iter().enumerate() {
            let current = tips[index].clone();
            let mut text = Text::default();
            text.push_span(
                Span::from(format!(" {} ", current.key))
                    .bold()
                    .bg(Color::White)
                    .fg(self.theme.primary),
            );
            text.push_span(Span::from(" "));
            text.push_span(Span::from(current.label));
            let tip = Paragraph::new(text);
            tip.render(*part, buf);
        }
    }
}
