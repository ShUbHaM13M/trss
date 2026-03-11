use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Flex, Layout},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    app::{AppCommand, Screens},
    event::Event,
    models::db::Database,
    screens::{Screen, ScreenContext, ScreenContextMut},
    utils::parse_html::{ParagraphList, parse_html},
};

pub struct ViewFeed {
    database: Database,
    content_list: ParagraphList,
    last_feed_id: Option<String>,
}

impl ViewFeed {
    pub fn new(database: Database) -> Self {
        Self {
            database,
            content_list: ParagraphList::new(vec![]),
            last_feed_id: None,
        }
    }

    fn sync_with_state(&mut self, ctx: &ScreenContextMut) {
        let selected_id = match ctx.state.selected_feed_item_id.as_ref() {
            Some(id) => id,
            None => return,
        };

        if self.last_feed_id.as_ref() == Some(selected_id) {
            return;
        }

        let mut found_item = None;

        // FIX: This is currently looping through all feed items, which is inefficient.
        // Although this happens only once when the feed id changes
        for collection in ctx.state.feed_items.values() {
            if let Some(&index) = collection.index_map.get(selected_id) {
                found_item = Some(&collection.items[index]);
                break;
            }
        }

        if let Some(item) = found_item {
            if let Some(content) = item.content.as_ref() {
                self.content_list
                    .set_paragraphs(parse_html(content.clone()));
            } else {
                self.content_list.set_paragraphs(vec![]);
            }

            self.content_list.reset_scroll();
            self.last_feed_id = Some(selected_id.clone());
        }
    }
}

impl Screen for ViewFeed {
    fn render(
        &self,
        frame: &mut ratatui::Frame<'_>,
        area: ratatui::prelude::Rect,
        ctx: &ScreenContext,
    ) {
        let theme = &ctx.config.current_theme;
        let selected_feed_id = match ctx.state.selected_feed_item_id.as_ref() {
            Some(item) => item,
            None => return,
        };

        let mut found_item = None;

        for collection in ctx.state.feed_items.values() {
            if let Some(&index) = collection.index_map.get(selected_feed_id) {
                found_item = Some(&collection.items[index]);
                break;
            }
        }

        if let Some(selected_item) = found_item {
            let container = Block::default()
                .padding(Padding::symmetric(2, 1))
                .style(Style::default().bg(theme.background));

            let inner_area = container.inner(area);

            let [_, inner_area, _] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Fill(1),
                    Constraint::Percentage(70),
                    Constraint::Fill(1),
                ])
                .areas(inner_area);

            let [header_area, content_area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Max(5), Constraint::Fill(1)])
                .margin(1)
                .spacing(1)
                .areas(inner_area);

            let [title_area, subtitle_area] = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Fill(1), Constraint::Fill(2)])
                .areas(header_area);

            let [title_area, favourite_area] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::SpaceBetween)
                .constraints([Constraint::Fill(1), Constraint::Max(2)])
                .areas(title_area);

            let title = Paragraph::new(selected_item.title.as_str())
                .bold()
                .fg(theme.primary);

            let subtitle = Paragraph::new(selected_item.summary.as_str())
                .wrap(Wrap { trim: true })
                .fg(theme.text);

            let favourite = Paragraph::new(Text::from(if selected_item.is_favourite {
                ""
            } else {
                ""
            }))
            .fg(theme.primary);

            title.render(title_area, frame.buffer_mut());
            favourite.render(favourite_area, frame.buffer_mut());
            subtitle.render(subtitle_area, frame.buffer_mut());

            frame.render_widget(container, area);

            self.content_list.render_ref(
                content_area,
                frame.buffer_mut(),
                &ctx.config.current_theme,
            );
        }
    }

    fn handle_input(&mut self, event: &Event, ctx: &ScreenContextMut) {
        self.sync_with_state(&ctx);

        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match key.code {
                    KeyCode::Char('s') => {
                        let _ = ctx
                            .command_tx
                            .send(AppCommand::Navigate(Screens::Home, None));
                    }
                    KeyCode::Char('j') | KeyCode::Down => {
                        self.content_list.scroll_down();
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        self.content_list.scroll_up();
                    }
                    KeyCode::Char('g') => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.content_list.scroll_bottom();
                        } else {
                            self.content_list.reset_scroll();
                        }
                    }
                    KeyCode::F(1) | KeyCode::Char('f') => {
                        let _ = ctx.command_tx.send(AppCommand::ToggleFeedItemFavourite(
                            self.last_feed_id.clone().unwrap(),
                        ));
                    }
                    _ => {}
                };
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.content_list.paragraphs.clear();
        self.content_list.reset_scroll();
        self.last_feed_id = None;
    }
}
