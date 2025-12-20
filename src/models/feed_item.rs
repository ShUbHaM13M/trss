use async_sqlite::rusqlite::Row;
use rss::Item;

use crate::models::db::Database;
use std::error::Error;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FeedItem {
    pub id: String,
    pub feed_id: i32,
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
}

impl FeedItem {
    fn from_row(row: &Row) -> Result<Self, async_sqlite::rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            feed_id: row.get(1)?,
            link: row.get(2)?,
            title: row.get(3)?,
            summary: row.get(4)?,
            content: row.get(5)?,
        })
    }

    pub async fn get_by_feed_id(
        feed_id: i32,
        database: &Database,
    ) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut feed_items =
            database
                .pool
                .conn(move |conn| {
                    let mut stmt = conn.prepare("SELECT id, feed_id, link, title, summary, content FROM feed_items WHERE feed_id = ?1")?;
                    let rows = stmt.query_map([feed_id], |row| FeedItem::from_row(row))?;
                    let mut feed_items: Vec<Self> = vec![];
                    for row in rows {
                        feed_items.push(row?);
                    }
                    Ok(feed_items)
                }).await?;

        let favourite_items = FeedItem::get_favourites(database).await?;
        let read_list_items = FeedItem::get_read_list_items(database).await?;

        feed_items.extend(favourite_items);
        feed_items.extend(read_list_items);

        Ok(feed_items)
    }

    pub async fn get_feed_item_count(
        feed_id: i32,
        database: &Database,
    ) -> Result<i32, Box<dyn Error>> {
        let count = database
            .pool
            .conn(move |conn| {
                let mut stmt =
                    conn.prepare("SELECT COUNT(*) FROM feed_items WHERE feed_id = ?1")?;
                let row = stmt.query_row([feed_id], |row| row.get(0))?;
                Ok(row)
            })
            .await?;

        Ok(count)
    }

    async fn get_favourites(database: &Database) -> Result<Vec<Self>, Box<dyn Error>> {
        let feed_items = database
            .pool
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    r"
                            SELECT FI.id, FI.feed_id, FI.link, FI.title, FI.summary, FI.content
                            FROM favourite_feeds FF
                            JOIN feed_items FI ON FF.feed_item_id = FI.id
                       ",
                )?;
                let rows = stmt.query_map([], |row| FeedItem::from_row(row))?;
                let mut feed_items: Vec<Self> = vec![];
                for row in rows {
                    let mut feed_item = row?;
                    feed_item.id = String::from("favourite");
                    feed_items.push(feed_item);
                }
                Ok(feed_items)
            })
            .await?;

        Ok(feed_items)
    }
    async fn get_read_list_items(database: &Database) -> Result<Vec<Self>, Box<dyn Error>> {
        let feed_items = database
            .pool
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    r"
                            SELECT FI.id, FI.feed_id, FI.link, FI.title, FI.summary, FI.content
                            FROM readlist R
                            JOIN feed_items FI ON R.feed_item_id = FI.id
                       ",
                )?;
                let rows = stmt.query_map([], |row| FeedItem::from_row(row))?;
                let mut feed_items: Vec<Self> = vec![];
                for row in rows {
                    let mut feed_item = row?;
                    feed_item.id = String::from("readlist");
                    feed_items.push(feed_item);
                }
                Ok(feed_items)
            })
            .await?;

        Ok(feed_items)
    }

    pub async fn update_feed_items_from_channel(
        feed_id: i32,
        items: &Vec<Item>,
        database: &Database,
    ) {
        const TRUNCATE_FEED_ITEM: &str = "TRUNCATE TABLE feed_items WHERE feed_id = ?1";
        const FEED_ITEM_INSERT: &str = r"
            INSERT INTO feed_items (id, feed_id, link, title, summary, content, published)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                link = excluded.link,
                title = excluded.title,
                summary = excluded.summary,
                content = excluded.content,
                published = excluded.published
        ";
        let _ = database
            .pool
            .conn(move |conn| {
                conn.execute(TRUNCATE_FEED_ITEM, &[&feed_id])?;
                Ok(())
            })
            .await;
        for item in items {
            let id = match item.guid.clone() {
                Some(guid) => guid.value.clone(),
                _ => match item.link.clone() {
                    Some(link) => link.clone(),
                    _ => String::new(),
                },
            };
            let link = match item.link.clone() {
                Some(link) => link.clone(),
                _ => String::new(),
            };
            let title = match item.title.clone() {
                Some(title) => title.clone(),
                _ => String::new(),
            };
            let summary = match item.description.clone() {
                Some(summary) => summary.clone(),
                _ => String::new(),
            };
            let content = match item.content.clone() {
                Some(content) => content,
                _ => String::new(),
            };
            let published = match item.pub_date.clone() {
                Some(pub_date) => pub_date,
                _ => String::new(),
            };
            // TODO: Remove after debugging
            let result = database
                .pool
                .conn(move |conn| {
                    conn.execute(
                        FEED_ITEM_INSERT,
                        [
                            id,
                            feed_id.to_string(),
                            link,
                            title,
                            summary,
                            content,
                            published,
                        ],
                    )?;
                    Ok(())
                })
                .await;

            // TODO: Remove after debugging
            if let Err(result) = result {
                println!("Error updating feed items {}", result);
            }
        }
    }
}
