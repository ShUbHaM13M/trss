use crate::app::App;

pub mod app;
pub mod event;
pub mod models;
pub mod screens;
pub mod utils;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let mut app = App::new().await;
    terminal.clear()?;
    let result = app.run(terminal);
    ratatui::restore();
    result
}
