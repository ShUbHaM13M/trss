use ratatui::{Frame, prelude::Rect};

use crate::{
    app::{App, AppEvent, AppState},
    event::EventHandler,
};

pub mod home;
pub mod test_html;
pub mod view_feed;

pub trait Screen {
    fn render(&self, frame: &mut Frame<'_>, area: Rect, app: &App);
    fn handle_input(&mut self, event: &EventHandler, state: &AppState) -> Option<AppEvent>;
    fn reset(&mut self);
}
