use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Widget, WidgetRef, Wrap},
};

use crate::{
    app::{AppEvent, AppState, Screens},
    event::Event,
    models::{db::Database, feed_item::FeedItem},
    screens::Screen,
    utils::parse_html::{ParagraphList, parse_html},
};

pub struct ViewFeed<'a> {
    database: Database,
    feed_item: Option<FeedItem>,
    previous_feed_item_id: Option<String>,
    content_list: ParagraphList<'a>,
}

impl<'a> ViewFeed<'a> {
    pub fn new(database: Database) -> Self {
        ViewFeed {
            database,
            feed_item: None,
            previous_feed_item_id: None,
            content_list: ParagraphList::new(vec![]),
        }
    }
    pub fn set_feed_item(&mut self, feed_item: FeedItem) {
        self.feed_item = Some(feed_item);
        if let Some(feed_item) = self.feed_item.as_ref() {
            if let Some(content) = feed_item.content.as_ref() {
                self.content_list
                    .set_paragraphs(parse_html(content.clone()));
            }
        }
    }
}

impl<'a> Screen for ViewFeed<'a> {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        app: &crate::app::App,
    ) {
        let selected_feed_item = app.state.selected_feed_item.as_ref().unwrap();

        let container = Block::default()
            .style(Style::default().fg(Color::White))
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = container.inner(area);

        let [header_area, content_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(10), Constraint::Fill(1)])
            .margin(1)
            .areas(inner_area);

        let header = Paragraph::new(selected_feed_item.summary.as_str())
            .block(Block::new().title(Span::styled(
                selected_feed_item.title.as_str(),
                Style::default().bold().underlined(),
            )))
            .wrap(Wrap { trim: true });

        frame.render_widget(container, area);
        header.render(header_area, frame.buffer_mut());

        // TODO: Update content_area to have a max_width and centered
        self.content_list
            .render_ref(content_area, frame.buffer_mut());
    }

    fn handle_input(
        &mut self,
        event: &crate::event::EventHandler,
        state: &AppState,
    ) -> Option<AppEvent> {
        if self.feed_item.is_none() {
            if let Some(feed_item) = state.selected_feed_item.as_ref() {
                self.set_feed_item(feed_item.clone());
                self.previous_feed_item_id = Some(feed_item.id.clone());
            }
        } else {
            if let Some(feed_item) = state.selected_feed_item.as_ref() {
                if let Some(previous_feed_item_id) = self.previous_feed_item_id.as_ref() {
                    if previous_feed_item_id != &feed_item.id {
                        self.set_feed_item(feed_item.clone());
                        self.previous_feed_item_id = Some(feed_item.id.clone());
                    }
                } else {
                    self.set_feed_item(feed_item.clone());
                    self.previous_feed_item_id = Some(feed_item.id.clone());
                }
            }
        }
        let event = event.next().unwrap();
        match event {
            Event::Tick => None,
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            return Some(AppEvent::Quit);
                        }
                        KeyCode::Char('s') => {
                            return Some(AppEvent::ChangeScreen(Screens::Home, state.clone()));
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.content_list.scroll_down();
                            return None;
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.content_list.scroll_up();
                            return None;
                        }
                        _ => None::<AppEvent>,
                    };
                }
                None
            }
            Event::Mouse(_) => None,
            Event::Resize(_, _) => None,
        }
    }
    fn reset(&mut self) {
        self.content_list.paragraphs = vec![];
        self.content_list.reset_scroll();
    }
}
