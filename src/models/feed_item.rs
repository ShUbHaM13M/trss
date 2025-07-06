use async_sqlite::rusqlite::Row;

use crate::models::db::Database;
use std::error::Error;

#[derive(Clone)]
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
}
