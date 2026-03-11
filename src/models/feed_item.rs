use async_sqlite::rusqlite::Row;
use rss::Item;

use crate::models::db::Database;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

const FEED_ITEM_INSERT: &str = r"
    INSERT INTO feed_items (id, link, title, summary, content, published, feed_id)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    ON CONFLICT(id) DO UPDATE SET
        link = excluded.link,
        title = excluded.title,
        summary = excluded.summary,
        content = excluded.content,
        published = excluded.published
";

#[repr(usize)]
enum FeedItemFields {
    Id = 0,
    FeedId,
    Link,
    Title,
    Summary,
    Content,
    IsFavourite,
    InReadlist,
}

#[derive(Debug, Clone)]
pub struct FeedItemCollection {
    pub items: Vec<FeedItem>,
    pub index_map: HashMap<String, usize>,
}

impl FeedItemCollection {
    pub fn new() -> Self {
        Self {
            items: vec![],
            index_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FeedItem {
    pub id: String,
    pub feed_id: i32,
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: Option<String>,
    pub is_favourite: bool,
    pub in_readlist: bool,
}

impl FeedItem {
    fn from_row(row: &Row) -> Result<Self, async_sqlite::rusqlite::Error> {
        let is_favourite: i32 = row
            .get(FeedItemFields::IsFavourite as usize)
            .unwrap_or_default();
        let is_favourite = if is_favourite == 0 { false } else { true };

        let in_readlist: i32 = row
            .get(FeedItemFields::InReadlist as usize)
            .unwrap_or_default();
        let in_readlist = if in_readlist == 0 { false } else { true };
        Ok(Self {
            id: row.get(FeedItemFields::Id as usize)?,
            feed_id: row.get(FeedItemFields::FeedId as usize)?,
            link: row.get(FeedItemFields::Link as usize)?,
            title: row.get(FeedItemFields::Title as usize)?,
            summary: row.get(FeedItemFields::Summary as usize)?,
            content: row.get(FeedItemFields::Content as usize)?,
            is_favourite, // row.get(6)?,
            in_readlist,  // row.get(7)?,
        })
    }

    pub async fn get_by_feed_id(
        feed_id: i32,
        database: &Database,
    ) -> Result<FeedItemCollection, Box<dyn Error>> {
        let feed_items = database
            .pool
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    r"
                        select
                            fi.id, fi.feed_id, fi.link, fi.title, fi.summary, fi.content,
                            case
                                when ff.feed_item_id is not null
                                then 1
                                else 0
                            end as is_favourite,
                            case
                                when r.feed_item_id is not null
                                then 1
                                else 0
                            end as in_readlist
                        from feed_items fi
                        left join favourite_feeds ff
                            on ff.feed_item_id = fi.id
                        left join readlist r
                            on r.feed_item_id  = fi.id
                        where fi.feed_id = ?1;
                    ",
                )?;
                let rows = stmt.query_map([feed_id], |row| FeedItem::from_row(row))?;
                let mut items: Vec<FeedItem> = vec![];
                let mut index_map: HashMap<String, usize> = HashMap::new();
                for (index, row) in rows.enumerate() {
                    if let Ok(row) = row {
                        index_map.insert(row.id.clone(), index);
                        items.push(row);
                    }
                }
                Ok(FeedItemCollection { items, index_map })
            })
            .await?;

        // let favourite_items: Vec<FeedItem> = feed_items
        //     .iter()
        //     .filter(|feed| feed.is_favourite)
        //     .cloned()
        //     .collect();

        // let read_list_items = FeedItem::get_read_list_items(database).await?;

        // feed_items.extend(favourite_items);
        // feed_items.extend(read_list_items);

        Ok(feed_items)
    }

    pub async fn get_feed_item_count(feed_id: i32, database: &Database) -> i32 {
        match database
            .pool
            .conn(move |conn| {
                let mut stmt =
                    conn.prepare("SELECT COUNT(*) FROM feed_items WHERE feed_id = ?1")?;
                let row = stmt.query_row([feed_id], |row| row.get(FeedItemFields::Id as usize))?;
                Ok(row)
            })
            .await
        {
            Ok(count) => count,
            Err(err) => {
                println!("Error: {}", err);
                0
            }
        }
    }

    pub async fn get_favourites_id(database: &Database) -> Result<HashSet<String>, Box<dyn Error>> {
        let feed_items = database
            .pool
            .conn(move |conn| {
                let mut stmt = conn.prepare(
                    r"
                        SELECT FI.id
                        FROM feed_items FI
                        JOIN favourite_feeds FF
                        ON FF.feed_item_id = FI.id;
                    ",
                )?;
                let rows = stmt.query_map([], |row| row.get(0))?;
                let mut favourites: HashSet<String> = HashSet::new();
                for row in rows {
                    favourites.insert(row?);
                }
                Ok(favourites)
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

    pub async fn get_feed_items_count(database: &Database) -> Result<i32, Box<dyn Error>> {
        let feed_item_count = database
            .pool
            .conn(move |conn| {
                let mut stmt = conn.prepare("SELECT COUNT(*) FROM feed_items")?;
                let count: i32 = stmt.query_one((), |row| row.get(FeedItemFields::Id as usize))?;
                Ok(count)
            })
            .await?;

        Ok(feed_item_count)
    }

    pub async fn update_feed_items_from_channel(
        feed_id: i32,
        items: Vec<Item>,
        database: &Database,
    ) {
        const DELETE_FEED_ITEM: &str = "DELETE FROM feed_items WHERE feed_id = ?1";
        let result = database
            .pool
            .conn(move |conn| {
                conn.execute_batch("BEGIN TRANSACTION;")?;
                conn.execute(DELETE_FEED_ITEM, ())?;
                let feed_id_str = feed_id.to_string();
                {
                    let mut stmt = conn.prepare(FEED_ITEM_INSERT)?;

                    for item in items {
                        let params = FeedItem::get_feed_items_param(&item, feed_id_str.clone());
                        stmt.execute(params)?;
                    }
                }
                conn.execute_batch("COMMIT;")?;
                Ok(())
            })
            .await;

        if let Err(e) = result {
            eprintln!("Error updating feed items: {}", e);
        }
    }

    fn get_feed_items_param(item: &Item, feed_id: String) -> [String; 7] {
        let id = item
            .guid
            .as_ref()
            .map(|g| g.value.clone())
            .or_else(|| item.link.clone())
            .unwrap_or_default();

        let link = item.link.clone().unwrap_or_default();
        let title = item.title.clone().unwrap_or_default();
        let summary = item.description.clone().unwrap_or_default();
        let content = item.content.clone().unwrap_or_default();
        let published = item.pub_date.clone().unwrap_or_default();
        [id, link, title, summary, content, published, feed_id]
    }

    pub async fn insert_feed_items_from_channel(
        feed_id: i32,
        items: Vec<Item>,
        database: &Database,
    ) -> Result<(), Box<dyn Error>> {
        let _ = database
            .pool
            .conn(move |conn| {
                conn.execute_batch("BEGIN TRANSACTION;")?;
                let feed_id_str = feed_id.to_string();
                {
                    let mut stmt = conn.prepare(FEED_ITEM_INSERT)?;

                    for item in items {
                        let params = FeedItem::get_feed_items_param(&item, feed_id_str.clone());
                        stmt.execute(params)?;
                    }
                }
                conn.execute_batch("COMMIT;")?;
                Ok(())
            })
            .await?;

        Ok(())
    }

    pub async fn toggle_favourite(feed_item_id: String, database: &Database) -> bool {
        const REMOVE_FAVOURITE: &str = r"
            DELETE FROM favourite_feeds
            WHERE feed_item_id = ?1;
        ";
        const INSERT_FAVOURITE: &str = r"
           INSERT INTO favourite_feeds (feed_item_id) VALUES (?1);
        ";
        let result = database
            .pool
            .conn(move |conn| {
                let mut remove_favourite = conn.prepare(REMOVE_FAVOURITE)?;
                let res = remove_favourite.execute((&feed_item_id,))?;
                if res == 0 {
                    let mut insert_favourite = conn.prepare(INSERT_FAVOURITE)?;
                    let res = insert_favourite.execute((&feed_item_id,))?;
                    return Ok(res == 1);
                }
                Ok(false)
            })
            .await;

        match result {
            Ok(result) => result,
            Err(err) => {
                eprintln!("{}", err);
                return false;
            }
        }
    }
}
