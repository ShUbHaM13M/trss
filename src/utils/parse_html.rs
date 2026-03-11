use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation,
    ScrollbarState, StatefulWidget, Widget, Wrap,
};
use tl::{Node, Parser};

use std::collections::HashMap;

use crate::models::theme::Theme;

lazy_static! {
    static ref HTML_ENTITIES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("&amp;", "&");
        m.insert("&#39;", "'");
        m.insert("&lt;", "<");
        m.insert("&gt;", ">");
        m.insert("&quot;", "\"");
        m.insert("&apos;", "'");
        m.insert("&nbsp;", " ");
        m.insert("&copy;", "©");
        m.insert("&reg;", "®");
        m.insert("&trade;", "™");
        m.insert("&euro;", "€");
        m.insert("&pound;", "£");
        m.insert("&yen;", "¥");
        m.insert("&sect;", "§");
        m.insert("&bull;", "•");
        m.insert("&hellip;", "…");
        m.insert("&mdash;", "—");
        m.insert("&ndash;", "–");
        m.insert("&tilde;", "˜");
        m.insert("&deg;", "°");
        m.insert("&plusmn;", "±");
        m
    };
    static ref CODE_STYLE: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("c", "&");
        m
    };
}

pub fn parse_html(html: String) -> Vec<ParagraphData> {
    let dom = tl::parse(&html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let mut text = Vec::new();

    for node_handle in dom.children().iter().filter_map(|h| h.get(parser)) {
        let parsed = parse_node(node_handle, parser);
        text.extend(parsed);
    }

    text
}

fn parse_inline<'a>(node: &Node, parser: &Parser) -> Vec<Span<'a>> {
    match node {
        Node::Raw(raw) => {
            let mut text = raw.as_utf8_str().to_string();
            for (key, value) in HTML_ENTITIES.iter() {
                text = text.replace(key, value);
            }
            vec![Span::raw(text)]
        }
        Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();
            let tag_name = tag_name.as_ref();

            let mut spans = Vec::new();
            for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                let child_spans = parse_inline(child, parser);
                spans.extend(child_spans);
            }

            let styled_spans = spans
                .into_iter()
                .map(|s| match tag_name {
                    "strong" | "b" => s.style(Style::default().bold()),
                    "em" | "i" => s.style(Style::default().italic()),
                    "u" => s.style(Style::default().underlined()),
                    "s" | "del" => s.style(Style::default().crossed_out()),
                    "code" => {
                        let content = s.content.clone();
                        let s = s
                            .content(format!("{}", content))
                            .style(Style::default().fg(Color::LightGreen).bold().reversed());
                        s
                    }
                    "li" => {
                        let content = s.content.clone();
                        s.content(format!(" # {}", content))
                    }
                    _ => s,
                })
                .collect();

            styled_spans
        }
        _ => vec![],
    }
}

fn parse_block<'a>(node: &Node, parser: &Parser) -> Vec<Line<'a>> {
    match node {
        Node::Raw(raw) => {
            let mut text = raw.as_utf8_str().to_string();
            for (key, value) in HTML_ENTITIES.iter() {
                text = text.replace(key, value);
            }
            vec![Line::raw(text)]
        }
        Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();
            let tag_name = tag_name.as_ref();

            let mut spans = Vec::new();
            for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                let child_spans = parse_inline(child, parser);
                spans.extend(child_spans);
            }

            let styled_spans = spans
                .into_iter()
                .map(|s| match tag_name {
                    "strong" | "b" => Line::from(s.style(Style::default().bold())),
                    "em" | "i" => Line::from(s.style(Style::default().italic())),
                    "u" => Line::from(s.style(Style::default().underlined())),
                    "s" | "del" => Line::from(s.style(Style::default().crossed_out())),
                    "code" => {
                        let content = s.content.clone();
                        let s = s
                            .content(content)
                            .style(Style::default().fg(Color::LightGreen).bold().reversed());
                        Line::from(s)
                    }
                    "li" => {
                        let content = s.content.clone();
                        Line::from(s.content(content))
                    }
                    _ => Line::from(s),
                })
                .collect();

            styled_spans
        }
        _ => vec![],
    }
}

fn parse_node<'a>(node: &Node, parser: &Parser) -> Vec<ParagraphData> {
    match node {
        Node::Tag(tag) => {
            let tag_name = tag.name().as_utf8_str();
            let tag_name = tag_name.as_ref();

            let mut inner_text = tag.inner_text(parser).to_string();
            for (key, value) in HTML_ENTITIES.iter() {
                inner_text = inner_text.replace(key, value);
            }

            match tag_name {
                "p" | "div" | "h3" | "h2" => {
                    let mut spans: Vec<Span> = vec![];

                    for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                        spans.extend(parse_inline(child, parser));
                    }

                    let mut lines: Vec<Line> = vec![];
                    let mut current_line: Vec<Span> = vec![];

                    for span in spans {
                        let parts = span.content.lines().collect::<Vec<_>>();
                        for (i, part) in parts.iter().enumerate() {
                            if i > 0 {
                                lines.push(Line::from(std::mem::take(&mut current_line)));
                            }

                            if !part.is_empty() {
                                current_line.push(Span::styled((*part).to_string(), span.style));
                            }
                        }
                    }

                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line));
                    }

                    vec![ParagraphData::new(Text::from(lines), None)]
                }
                "strong" | "b" => {
                    vec![ParagraphData::new(
                        Span::from(inner_text).style(Style::default().bold()),
                        None,
                    )]
                }
                "em" | "i" => {
                    vec![ParagraphData::new(
                        Span::from(inner_text).style(Style::default().italic()),
                        None,
                    )]
                }
                "u" => {
                    vec![ParagraphData::new(
                        Span::from(inner_text).style(Style::default().underlined()),
                        None,
                    )]
                }
                "s" | "del" => {
                    vec![ParagraphData::new(
                        Span::from(inner_text).style(Style::default().crossed_out()),
                        None,
                    )]
                }
                "code" => {
                    let mut filename = String::from(" code ");
                    if inner_text.starts_with("~filename") {
                        let mut lines = inner_text.lines();
                        if let Some(line) = lines.next() {
                            filename = format!(" {} ", line.replace("~filename", ""));
                        }
                        inner_text = lines.collect::<Vec<_>>().join("\n");
                    }
                    let content = inner_text
                        .lines()
                        .map(|line| {
                            // TODO: Here we can add syntax highlighting
                            Line::from(Span::styled(
                                line.to_string(),
                                Style::default().fg(Color::Black).bg(Color::LightGreen),
                            ))
                        })
                        .collect::<Vec<Line>>();

                    let block = Block::default()
                        .title(filename)
                        .style(Style::default().fg(Color::Black))
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Black))
                        .padding(Padding::horizontal(1))
                        .bg(Color::LightGreen);

                    vec![ParagraphData::new(content, Some(block))]
                }
                "ul" => {
                    let mut lines: Vec<Line> = vec![];
                    for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                        lines.extend(
                            parse_block(child, parser)
                                .iter()
                                .filter(|line| line.spans.iter().any(|s| s.content.len() > 0))
                                .map(|line| {
                                    let mut ls = Line::from("");
                                    for l in line.iter() {
                                        ls.push_span(Span::from("⦿  "));
                                        ls.push_span(l.clone());
                                    }
                                    ls
                                }),
                        );
                    }
                    vec![ParagraphData::new(Text::from(lines), None)]
                }
                "ol" => {
                    let mut lines: Vec<Line> = vec![];
                    let mut index = 1;
                    for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                        lines.extend(parse_block(child, parser).iter().map(|line| {
                            let mut ls = Line::from("");
                            for l in line.iter() {
                                ls.push_span(Span::from(format!("{}. ", index)));
                                ls.push_span(l.clone());
                                index += 1;
                            }
                            ls
                        }));
                    }
                    vec![ParagraphData::new(Text::from(lines), None)]
                }
                "blockquote" => {
                    let block = Block::default()
                        .style(Style::default().fg(Color::Black))
                        .borders(Borders::LEFT)
                        .border_type(BorderType::Double)
                        .border_style(Style::default().fg(Color::Black))
                        .padding(Padding::uniform(1))
                        .bg(Color::LightCyan);
                    vec![ParagraphData::new(Span::from(inner_text), Some(block))]
                }
                "hr" => {
                    let block = Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(Color::White))
                        .border_type(BorderType::Thick);
                    vec![ParagraphData::new(Span::from(""), Some(block))]
                }
                "pre" | "video" | "source" => {
                    let mut children_widgets = Vec::new();
                    for child in tag.children().top().iter().filter_map(|h| h.get(parser)) {
                        children_widgets.extend(parse_node(child, parser));
                    }
                    children_widgets
                }
                _ => {
                    unimplemented!("{} not implemented", tag_name);
                }
            }
        }
        Node::Raw(raw) => {
            let mut text = raw.as_utf8_str().trim().to_string();
            for (key, value) in HTML_ENTITIES.iter() {
                text = text.replace(key, value);
            }
            let span = Span::raw(text);
            vec![ParagraphData::new(span, None)]
        }
        _ => vec![],
    }
}

#[derive(Clone, Debug)]
pub struct ParagraphData {
    text: Text<'static>,
    block: Option<Block<'static>>,
    alignment: Alignment,
    wrap: bool,
}

impl ParagraphData {
    pub fn new<T>(text: T, block: Option<Block<'static>>) -> Self
    where
        T: Into<Text<'static>>,
    {
        Self {
            text: text.into(),
            block,
            alignment: Alignment::Left,
            wrap: true,
        }
    }

    pub fn to_paragraph(&self) -> Paragraph<'_> {
        let mut p = Paragraph::new(self.text.clone()).alignment(self.alignment);

        if self.wrap {
            p = p.wrap(Wrap { trim: false });
        }

        if let Some(b) = &self.block {
            p = p.block(b.clone());
        }

        p
    }
}

pub struct ParagraphList {
    pub paragraphs: Vec<ParagraphData>,
    heights: Vec<u16>,
    scroll: usize,
    // content_height: usize,
}

impl ParagraphList {
    pub fn new(paragraphs: Vec<ParagraphData>) -> Self {
        Self {
            paragraphs,
            heights: vec![],
            scroll: 0,
        }
    }
    pub fn set_paragraphs(&mut self, paragraphs: Vec<ParagraphData>) {
        self.heights = paragraphs
            .iter()
            .map(|p| (p.text.lines.len() as u16).max(1))
            .collect();
        self.paragraphs = paragraphs;
        self.scroll = 0;
    }
    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
    pub fn scroll_bottom(&mut self) {
        // TODO: Need to find a way to scroll to bottom
        // self.scroll = self.scroll.saturating_add();
    }
    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }
    fn estimate_paragraph_height(p: &ParagraphData, area_width: u16) -> usize {
        let mut lines = p
            .text
            .lines
            .iter()
            .map(|line| {
                let w = line.width();
                (w as f32 / area_width as f32).ceil() as usize
            })
            .sum::<usize>();

        if p.block.is_some() {
            lines += 2;
        }

        lines
    }
    fn measure_text_lines(text: &Text<'static>, width: u16) -> u16 {
        if width == 0 {
            return 0;
        }
        let mut lines = 0usize;
        for line in text.lines.clone() {
            let mut len = 0usize;
            for span in line.iter() {
                len += span.content.len();
            }
            let wraps = (len + (width as usize).saturating_sub(1)) / width as usize;
            lines = lines.saturating_add(std::cmp::max(1, wraps));
        }
        lines as u16
    }

    fn block_vertical_extra(block: &Option<ratatui::widgets::Block<'static>>) -> u16 {
        if block.is_some() { 2 } else { 0 }
    }

    pub fn render_ref(&self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        let width = area.width.saturating_sub(2);
        if width == 0 || area.height == 0 {
            return;
        }

        let mut lines_to_skip = self.scroll as u32;
        let mut y_offset: u16 = 0;

        for pd in &self.paragraphs {
            let text_lines = Self::measure_text_lines(&pd.text, width) as u32;
            let extra = Self::block_vertical_extra(&pd.block) as u32;
            let par_lines = text_lines.saturating_add(extra);

            if lines_to_skip >= par_lines {
                lines_to_skip = lines_to_skip.saturating_sub(par_lines);
                continue;
            }

            let remaining_par_lines = par_lines.saturating_sub(lines_to_skip);
            let remaining_area = area.height.saturating_sub(y_offset) as u32;
            if remaining_area == 0 {
                break;
            }
            let draw_lines = std::cmp::min(remaining_par_lines, remaining_area) as u16;

            let skip = lines_to_skip as u16;
            let temp_h = skip.saturating_add(draw_lines);
            let mut temp_buf = Buffer::empty(Rect {
                x: 0,
                y: 0,
                width,
                height: temp_h,
            });

            let p = match pd.block.is_some() {
                true => pd.to_paragraph(),
                false => pd.to_paragraph().bg(theme.background).fg(theme.text),
            };
            p.render(
                Rect {
                    x: 0,
                    y: 0,
                    width,
                    height: temp_h,
                },
                &mut temp_buf,
            );

            for row in 0..draw_lines {
                let src_y = skip + row;
                let dst_y = area.y + y_offset + row;
                for col in 0..width {
                    let src = {
                        let this = &temp_buf;
                        let i = this.index_of(col, src_y);
                        &this.content[i]
                    };
                    {
                        let this = &mut *buf;
                        let x = area.x + col;
                        let i = this.index_of(x, dst_y);
                        &mut this.content[i]
                    }
                    .clone_from(src);
                }
            }

            y_offset = y_offset.saturating_add(draw_lines);
            lines_to_skip = 0;

            if y_offset >= area.height {
                break;
            }
        }

        let content_height: usize = self
            .paragraphs
            .iter()
            .map(|p| Self::estimate_paragraph_height(p, width))
            .sum();

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .begin_style(Style::default().bg(theme.background).fg(theme.text))
            .end_style(Style::default().bg(theme.background).fg(theme.text))
            .track_symbol(Some(" "))
            .track_style(Style::default().bg(theme.text))
            .thumb_style(Style::default().fg(theme.primary));

        let mut scrollbar_state = ScrollbarState::new(content_height).position(self.scroll);
        scrollbar.render(area, buf, &mut scrollbar_state);
    }
}
