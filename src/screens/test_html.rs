use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    app::{AppCommand, Screens},
    event::Event,
    screens::{Screen, ScreenContext, ScreenContextMut},
    utils::parse_html::{ParagraphList, parse_html},
};

pub struct TestHtml {
    list: ParagraphList,
}

impl TestHtml {
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

impl Screen for TestHtml {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        ctx: &ScreenContext,
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

        self.list
            .render_ref(content_area, frame.buffer_mut(), &ctx.config.current_theme);
    }

    fn handle_input(&mut self, event: &Event, ctx: &ScreenContextMut) {
        match event {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            let _ = ctx.command_tx.send(AppCommand::Quit);
                        }
                        KeyCode::Char('h') => {
                            let _ = ctx
                                .command_tx
                                .send(AppCommand::Navigate(Screens::Home, None));
                        }
                        KeyCode::Down => {
                            self.list.scroll_down();
                        }
                        KeyCode::Up => {
                            self.list.scroll_up();
                        }
                        _ => {}
                    };
                }
            }
            _ => {}
        }
    }
    fn reset(&mut self) {
        self.list.reset_scroll();
    }
}
