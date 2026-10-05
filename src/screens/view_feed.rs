use ratatui::{
    crossterm::event::KeyEventKind,
    layout::{Constraint, Direction, Flex, Layout},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph, Widget, Wrap},
};

use crate::{
    app::{AppCommand, AppState, Screens},
    event::Event,
    screens::{Screen, ScreenContext, ScreenContextMut},
    utils::parse_html::{ParagraphList, parse_html},
};

pub struct ViewFeed {
    content_list: ParagraphList,
    last_feed_id: Option<String>,
}

impl ViewFeed {
    pub fn new() -> Self {
        Self {
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

    fn move_article(
        state: &AppState,
        command_tx: &tokio::sync::mpsc::UnboundedSender<AppCommand>,
        forward: bool,
    ) {
        let Some(current_id) = state.selected_feed_item_id.as_ref() else {
            return;
        };
        let mut items = state.items_for_selected_source();
        if !items.iter().any(|item| &item.id == current_id) {
            items = state
                .feed_items
                .values()
                .flat_map(|collection| collection.items.iter().cloned())
                .collect();
            items.sort_by(|a, b| a.title.cmp(&b.title).then_with(|| a.id.cmp(&b.id)));
        }
        let Some(index) = items.iter().position(|item| &item.id == current_id) else {
            return;
        };
        if items.is_empty() {
            return;
        }
        let next = if forward {
            (index + 1) % items.len()
        } else if index == 0 {
            items.len() - 1
        } else {
            index - 1
        };
        let _ = command_tx.send(AppCommand::OpenFeed(items[next].id.clone()));
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

            let [title_area, flags_area] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::SpaceBetween)
                .constraints([Constraint::Fill(1), Constraint::Max(7)])
                .areas(title_area);

            let title = Paragraph::new(selected_item.title.as_str())
                .bold()
                .fg(theme.primary);

            let subtitle = Paragraph::new(selected_item.summary.as_str())
                .wrap(Wrap { trim: true })
                .fg(theme.text);

            let item_flags = Paragraph::new(format!(
                "F{} R{}",
                if selected_item.is_favourite { "+" } else { "-" },
                if selected_item.in_readlist { "+" } else { "-" },
            ))
            .fg(theme.primary);

            title.render(title_area, frame.buffer_mut());
            item_flags.render(flags_area, frame.buffer_mut());
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
                let keymap = &ctx.config.keymap;
                if keymap.matches("article", "back", *key) {
                    let _ = ctx
                        .command_tx
                        .send(AppCommand::Navigate(Screens::Home, None));
                } else if keymap.matches("article", "down", *key) {
                    self.content_list.scroll_down();
                } else if keymap.matches("article", "up", *key) {
                    self.content_list.scroll_up();
                } else if keymap.matches("article", "page_down", *key) {
                    self.content_list.scroll_by(20);
                } else if keymap.matches("article", "page_up", *key) {
                    for _ in 0..20 {
                        self.content_list.scroll_up();
                    }
                } else if keymap.matches("article", "top", *key) {
                    self.content_list.reset_scroll();
                } else if keymap.matches("article", "bottom", *key) {
                    self.content_list.scroll_bottom();
                } else if keymap.matches("article", "next", *key) {
                    Self::move_article(ctx.state, &ctx.command_tx, true);
                } else if keymap.matches("article", "previous", *key) {
                    Self::move_article(ctx.state, &ctx.command_tx, false);
                } else if keymap.matches("article", "favourite", *key) {
                    if let Some(id) = ctx.state.selected_feed_item_id.clone() {
                        let _ = ctx.command_tx.send(AppCommand::ToggleFeedItemFavourite(id));
                    }
                } else if keymap.matches("article", "readlist", *key) {
                    if let Some(id) = ctx.state.selected_feed_item_id.clone() {
                        let _ = ctx.command_tx.send(AppCommand::ToggleFeedItemReadlist(id));
                    }
                }
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
