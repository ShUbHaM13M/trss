use reqwest::get;
use rss::Channel;

use crate::app::AppEvent;
use crate::models::{db::Database, feed_item::FeedItem};
use crate::utils::DEFAULT_DATE_FORMAT;
use std::error::Error;

#[repr(usize)]
enum FeedFields {
    Id = 0,
    Title,
    SubTitle,
    Url,
    FeedCount,
}

#[derive(Debug, Clone)]
pub enum FeedSource {
    Feed(i32),
    Favourites,
    Readlist,
}

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
                let mut stmt = conn.prepare(
                    "SELECT id, title, subtitle, url FROM feeds WHERE active = 1 ORDER BY id DESC",
                )?;
                let mut rows = stmt.query([])?;
                let mut feeds: Vec<Feed> = vec![];
                while let Some(row) = rows.next()? {
                    feeds.push(Feed {
                        id: row.get(FeedFields::Id as usize)?,
                        title: row.get(FeedFields::Title as usize)?,
                        subtitle: row.get(FeedFields::SubTitle as usize)?,
                        url: row.get(FeedFields::Url as usize)?,
                        feed_count: 0,
                    })
                }
                Ok(feeds)
            })
            .await?;

        // feeds.push(Feed {
        //     id: -1,
        //     title: String::from("Favourites"),
        //     subtitle: String::from("Your favourite feeds"),
        //     ..Default::default()
        // });
        // feeds.push(Feed {
        //     id: -2,
        //     title: String::from("Readlist"),
        //     subtitle: String::from("Your readlist"),
        //     ..Default::default()
        // });

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

    pub async fn add(
        title: String,
        url: String,
        database: &Database,
    ) -> Result<Self, Box<dyn Error>> {
        const FEED_INSERT: &str = r"
        INSERT INTO feeds
            (title, subtitle, url, image, last_updated)
        VALUES (?1, ?2, ?3, ?4, ?5)
        RETURNING id;
        ";
        let response = get(url.clone()).await?;
        let body = response.bytes().await?;
        let channel = Channel::read_from(&body[..])?;

        let title = if title.is_empty() {
            channel.title.clone()
        } else {
            title
        };
        let subtitle = channel.description.clone();
        let image = match channel.image.clone() {
            Some(image) => image.url,
            None => String::new(),
        };
        let last_updated = match channel.last_build_date.clone() {
            Some(last_updated) => last_updated,
            _ => {
                let local_time = chrono::offset::Local::now();
                local_time.format(DEFAULT_DATE_FORMAT).to_string()
            }
        };

        let params = (
            title.clone(),
            subtitle.clone(),
            url.clone(),
            image.clone(),
            last_updated.clone(),
        );

        let id = database
            .pool
            .conn(move |conn| {
                let id = conn.query_one(FEED_INSERT, params, |row| {
                    let id: i32 = row.get(FeedFields::Id as usize)?;
                    Ok(id)
                })?;
                Ok(id)
            })
            .await?;

        let feed_count = channel.items.len() as i32;
        FeedItem::insert_feed_items_from_channel(id, channel.items, database).await?;

        return Ok(Self {
            id,
            title,
            subtitle,
            url,
            feed_count,
        });
    }

    pub async fn delete(id: i32, database: &Database) -> Result<(), Box<dyn Error>> {
        let _ = database
            .pool
            .conn(move |conn| {
                conn.execute("DELETE FROM feeds WHERE id = ?1", (id,))?;
                Ok(())
            })
            .await?;

        Ok(())
    }
}

pub fn spawn_update_feeds(feeds: Vec<Feed>, sender: tokio::sync::mpsc::UnboundedSender<AppEvent>) {
    let _ = sender.send(AppEvent::BackgroundSyncStarted);
    tokio::spawn(async move {
        let database = match Database::init().await {
            Ok(db) => db,
            Err(e) => {
                println!("Failed to initialize database: {}", e);
                return;
            }
        };

        let mut updated_feeds = Vec::new();

        for feed in feeds {
            if feed.url.is_empty() {
                continue;
            }

            match get(&feed.url).await {
                Ok(response) => match response.bytes().await {
                    Ok(body) => {
                        if let Ok(channel) = Channel::read_from(&body[..]) {
                            let feed_item_count =
                                FeedItem::get_feed_item_count(feed.id, &database).await;
                            if feed_item_count != channel.items.len() as i32 {
                                let _ = Feed::update_feed(&channel, &database).await;

                                let _ = FeedItem::update_feed_items_from_channel(
                                    feed.id,
                                    channel.items,
                                    &database,
                                )
                                .await;

                                updated_feeds.push(feed.id);
                            }
                        }
                    }
                    Err(_) => {
                        println!("Failed to read body for {}", feed.url);
                    }
                },
                Err(e) => {
                    println!("Failed to fetch {} {}", feed.url, e);
                }
            }
        }

        let _ = sender.send(AppEvent::FeedsUpdated(updated_feeds));
    });
}
