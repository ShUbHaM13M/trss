#[macro_use]
extern crate lazy_static;

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
    let mut app = match App::new().await {
        Ok(app) => app,
        Err(error) => {
            ratatui::restore();
            return Err(error);
        }
    };

    let result = match terminal.clear() {
        Ok(()) => app.run(terminal).await,
        Err(error) => Err(Box::new(error) as Box<dyn std::error::Error>),
    };
    ratatui::restore();
    result
}
