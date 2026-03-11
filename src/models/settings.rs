use std::error::Error;

use crate::models::db::Database;

#[derive(Debug)]
pub struct Settings {
    pub theme: String,
    pub background_sync: bool,
}

impl Settings {
    pub async fn init(database: &Database) -> Result<(), Box<dyn Error>> {
        database
            .pool
            .conn(|conn| {
                const INSERT_THEME: &str = r"
                INSERT OR IGNORE INTO settings
                    (name, value)
                VALUES (?, ?)
                ";
                let mut stmt = conn.prepare(INSERT_THEME)?;
                stmt.execute(("background_sync", false))?;
                stmt.execute(("theme", "Dark"))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn get_settings(database: &Database) -> Self {
        let settings = Self {
            theme: String::from("Dark"),
            background_sync: false,
        };
        database
            .pool
            .conn(|conn| {
                let mut stmt = conn.prepare("SELECT name, value FROM settings")?;
                let mut rows = stmt.query(())?;
                let mut settings = Self {
                    theme: String::from("Dark"),
                    background_sync: false,
                };

                while let Some(row) = rows.next()? {
                    let name: String = row.get(0)?;
                    let value: String = row.get(1)?;

                    match name.as_ref() {
                        "background_sync" => {
                            settings.background_sync = match value.as_ref() {
                                "1" => true,
                                _ => false,
                            };
                        }
                        "theme" => {
                            settings.theme = value;
                        }
                        _ => {}
                    }
                }

                Ok(settings)
            })
            .await
            .unwrap_or(settings)
    }

    pub async fn update_setting(
        database: &Database,
        name: String,
        value: String,
    ) -> Result<(), Box<dyn Error>> {
        database
            .pool
            .conn(|conn| {
                let mut stmt = conn.prepare("UPDATE settings SET value = ?1 WHERE name = ?2")?;
                let rows = stmt.execute((value, name))?;
                Ok(rows)
            })
            .await?;
        Ok(())
    }
}
