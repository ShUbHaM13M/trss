#[macro_use]
extern crate lazy_static;

use reqwest::get;
use rss::Channel;

use crate::app::App;
use crate::models::db::Database;
use crate::models::feed::Feed;
use crate::models::feed_item::FeedItem;

pub mod app;
pub mod event;
pub mod models;
pub mod screens;
pub mod utils;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if true {
        let mut terminal = ratatui::init();
        let mut app = App::new().await;
        terminal.clear()?;
        let result = app.run(terminal).await;
        ratatui::restore();
        result
    } else {
        let database = Database::init().await?;
        let response = get("https://shubham-maurya.vercel.app/rss.xml").await?;
        let body = response.bytes().await?;
        let channel = Channel::read_from(&body[..])?;
        FeedItem::insert_feed_items_from_channel(31, channel.items, &database).await?;

        // let mut _feeds: Vec<Feed> = Vec::new();
        // if let Ok(f) = Feed::get_all(&database).await {
        //     _feeds = f;
        // }
        // let (sender, _) = channel();
        // spawn_update_feeds(feeds.clone(), sender).await;
        Ok(())
    }
}
