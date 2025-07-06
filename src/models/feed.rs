use crate::models::db::Database;
use std::error::Error;

#[derive(Clone)]
pub struct Feed {
    pub id: i32,
    pub title: String,
    pub subtitle: String,
    pub url: String,
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
                    })
                }
                Ok(feeds)
            })
            .await?;

        feeds.push(Feed {
            id: feeds.len() as i32 + 1,
            title: String::from("Favourites"),
            subtitle: String::from("Your favourite feeds"),
            url: String::new(),
        });
        feeds.push(Feed {
            id: feeds.len() as i32 + 1,
            title: String::from("Readlist"),
            subtitle: String::from("Your readlist"),
            url: String::new(),
        });
        Ok(feeds)
    }
}
