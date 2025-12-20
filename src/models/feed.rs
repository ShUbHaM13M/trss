use reqwest::get;
use rss::Channel;
use std::thread;
use tokio::runtime::Runtime;

use crate::models::{db::Database, feed_item::FeedItem};
use crate::utils::DEFAULT_DATE_FORMAT;
use std::error::Error;
use std::sync::mpsc::{self};

#[derive(Clone, Default, Debug)]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub subtitle: String,
    pub url: String,
    pub feed_count: i32,
}

impl Feed {
    pub async fn get_all(database: &Database) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut feeds = database
            .pool
            .conn(|conn| {
                let mut stmt = conn.prepare("SELECT id, title, subtitle, url FROM feeds")?;
                let mut rows = stmt.query([])?;
                let mut feeds: Vec<Feed> = vec![];
                while let Some(row) = rows.next()? {
                    feeds.push(Feed {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        subtitle: row.get(2)?,
                        url: row.get(3)?,
                        feed_count: 0,
                    })
                }
                Ok(feeds)
            })
            .await?;

        feeds.push(Feed {
            id: feeds.len() as i32 + 1,
            title: String::from("Favourites"),
            subtitle: String::from("Your favourite feeds"),
            ..Default::default()
        });
        feeds.push(Feed {
            id: feeds.len() as i32 + 1,
            title: String::from("Readlist"),
            subtitle: String::from("Your readlist"),
            ..Default::default()
        });

        Ok(feeds)
    }

    pub async fn update_feed(channel: &Channel, database: &Database) {
        const FEED_UPDATE: &str = r"
        UPDATE feeds
            SET title = ?1, subtitle = ?2, last_updated = ?3
        WHERE url = ?4";
        let title: String = channel.title.clone();
        let description: String = channel.description.clone();
        let last_updated = match channel.last_build_date.clone() {
            Some(last_updated) => last_updated,
            _ => {
                let local_time = chrono::offset::Local::now();
                local_time.format(DEFAULT_DATE_FORMAT).to_string()
            }
        };
        let url = channel.link.clone();
        let result = database
            .pool
            .conn(move |conn| {
                conn.execute(FEED_UPDATE, &[&title, &description, &last_updated, &url])
            })
            .await;

        // TODO: Remove after debugging
        if let Err(result) = result {
            println!("Error updating feed {}", result);
        }
    }
}

pub async fn spawn_update_feeds(feeds: Vec<Feed>, sender: mpsc::Sender<Vec<i32>>) {
    thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let mut updated_feeds = Vec::new();

        rt.block_on(async {
            let database = Database::init().await.unwrap();
            for feed in feeds {
                if feed.url.is_empty() {
                    continue;
                }
                match get(&feed.url).await {
                    Ok(response) => {
                        if let Ok(body) = response.bytes().await {
                            let channel = Channel::read_from(&body[..]);
                            if let Ok(channel) = channel {
                                if let Ok(feed_item_count) =
                                    FeedItem::get_feed_item_count(feed.id, &database).await
                                {
                                    if feed_item_count != channel.items.len() as i32 {
                                        Feed::update_feed(&channel, &database).await;
                                        FeedItem::update_feed_items_from_channel(
                                            feed.id,
                                            &channel.items,
                                            &database,
                                        )
                                        .await;
                                        updated_feeds.push(feed.id);
                                    }
                                }
                            }
                        } else {
                            eprintln!("Failed to read body for {}", feed.url);
                        }
                    }
                    Err(_) => {
                        // eprintln!("Failed to fetch {}: {}", feed.url, e);
                    }
                }
            }

            let _ = sender.send(updated_feeds);
        });
    });
}
