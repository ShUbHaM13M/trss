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
        let feeds = database
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

    pub async fn update_feed(
        feed_id: i32,
        channel: &Channel,
        database: &Database,
    ) -> Result<(), Box<dyn Error>> {
        const FEED_UPDATE: &str = r"
        UPDATE feeds
            SET title = ?1, subtitle = ?2, last_updated = ?3
        WHERE id = ?4";
        let title: String = channel.title.clone();
        let description: String = channel.description.clone();
        let last_updated = match channel.last_build_date.clone() {
            Some(last_updated) => last_updated,
            _ => {
                let local_time = chrono::offset::Local::now();
                local_time.format(DEFAULT_DATE_FORMAT).to_string()
            }
        };
        database
            .pool
            .conn(move |conn| {
                conn.execute(FEED_UPDATE, (&title, &description, &last_updated, feed_id))?;
                Ok(())
            })
            .await?;
        Ok(())
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
        let response = get(url.clone()).await?.error_for_status()?;
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
                let _ = sender.send(AppEvent::FeedsUpdated(
                    Vec::new(),
                    vec![format!("Could not open the feed database: {e}")],
                ));
                return;
            }
        };

        let mut updated_feeds = Vec::new();
        let mut errors = Vec::new();

        for feed in feeds {
            if feed.url.is_empty() {
                continue;
            }

            let result = async {
                let response = get(&feed.url).await?.error_for_status()?;
                let body = response.bytes().await?;
                let channel = Channel::read_from(&body[..])?;
                Ok::<Channel, Box<dyn Error + Send + Sync>>(channel)
            }
            .await;

            match result {
                Ok(channel) => {
                    let item_update_error = match FeedItem::update_feed_items_from_channel(
                        feed.id,
                        channel.items.clone(),
                        &database,
                    )
                    .await
                    {
                        Ok(()) => None,
                        Err(error) => Some(error.to_string()),
                    };
                    if let Some(error) = item_update_error {
                        errors.push(format!(
                            "{}: could not update articles: {error}",
                            feed.title
                        ));
                        continue;
                    }

                    updated_feeds.push(feed.id);
                    if let Err(error) = Feed::update_feed(feed.id, &channel, &database).await {
                        errors.push(format!(
                            "{}: articles updated, metadata update failed: {error}",
                            feed.title
                        ));
                    }
                }
                Err(error) => errors.push(format!("{}: {error}", feed.title)),
            }
        }

        let _ = sender.send(AppEvent::FeedsUpdated(updated_feeds, errors));
    });
}
