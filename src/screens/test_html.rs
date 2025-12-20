use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders, WidgetRef},
};

use crate::{
    app::{AppEvent, Screens},
    event::Event,
    screens::Screen,
    utils::parse_html::{ParagraphList, parse_html},
};

pub struct TestHtml<'a> {
    list: ParagraphList<'a>,
}

impl<'a> TestHtml<'a> {
    pub fn new() -> Self {
        let text = parse_html(String::from(
            r#"
<ol><li>First</li><li>Second</li><li>Third</li><li>Fourth</li><li><b>Fifth</b></li></ol>
<ul><li>First</li><li>Second</li><li>Third</li><li>Fourth</li><li><b>Fifth</b></li></ul>
    "#,
        ));
        TestHtml {
            list: ParagraphList::new(text),
        }
    }
}

impl<'a> Screen for TestHtml<'a> {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        _app: &crate::app::App,
    ) {
        let container = Block::default()
            .title(Line::from("HTML").centered())
            .style(Style::default().fg(Color::White))
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = container.inner(area);
        let [content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1)])
            .margin(1)
            .areas(inner_area);

        self.list.render_ref(content_area, frame.buffer_mut());
    }

    fn handle_input(
        &mut self,
        event: &crate::event::EventHandler,
        state: &crate::app::AppState,
    ) -> Option<crate::app::AppEvent> {
        let event = event.next().unwrap();
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            return Some(AppEvent::Quit);
                        }
                        KeyCode::Char('h') => {
                            return Some(AppEvent::ChangeScreen(Screens::Home, state.clone()));
                        }
                        KeyCode::Down => {
                            self.list.scroll_down();
                            None
                        }
                        KeyCode::Up => {
                            self.list.scroll_up();
                            None
                        }
                        _ => None::<AppEvent>,
                    };
                }
                None
            }
            _ => None,
        }
    }
    fn reset(&mut self) {
        self.list.reset_scroll();
    }
}
