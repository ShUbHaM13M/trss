use ratatui::{Frame, prelude::Rect};
use tokio::sync::mpsc;

use crate::{
    app::{AppCommand, AppConfig, AppEvent, AppState},
    event::Event,
};

pub mod home;
pub mod test_html;
pub mod view_feed;

pub struct ScreenContext<'a> {
    pub state: &'a AppState,
    pub config: &'a AppConfig,
}

pub struct ScreenContextMut<'a> {
    pub state: &'a mut AppState,
    pub config: &'a mut AppConfig,
    pub event_tx: mpsc::UnboundedSender<AppEvent>,
    pub command_tx: mpsc::UnboundedSender<AppCommand>,
}

pub trait Screen {
    fn render(&self, frame: &mut Frame<'_>, area: Rect, ctx: &ScreenContext);
    fn handle_input(&mut self, event: &Event, ctx: &ScreenContextMut);
    fn reset(&mut self);
}
