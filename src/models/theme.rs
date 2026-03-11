use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::models::db::Database;

const USER_THEMES_FILE: &str = "themes.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Theme {
    pub primary: Color,
    pub background: Color,
    pub text: Color,
    pub border: Color,
}

impl Theme {
    pub fn new(primary: Color, background: Color, text: Color, border: Color) -> Self {
        Self {
            primary,
            background,
            text,
            border,
        }
    }
}

pub static LIGHT_THEME: Theme = Theme {
    primary: Color::from_u32(0x1152d4),
    background: Color::from_u32(0xf6f6f8),
    text: Color::from_u32(0x92a4c9),
    border: Color::from_u32(0x232f48),
};

pub static DARK_THEME: Theme = Theme {
    primary: Color::from_u32(0x1152d4),
    background: Color::from_u32(0x0a0a0a),
    text: Color::from_u32(0x92a4c9),
    border: Color::from_u32(0x232f48),
};

impl Theme {
    pub async fn init(database: &Database) -> Result<(), Box<dyn Error>> {
        database
            .pool
            .conn(|conn| {
                const INSERT_THEME: &str = r"
                INSERT OR IGNORE INTO themes
                    (name, primary_color, background_color, text_color, border_color)
                VALUES (?, ?, ?, ?, ?)
                ";
                let mut stmt = conn.prepare(INSERT_THEME)?;
                stmt.execute(("light", 0x1152d4, 0xf6f6f8, 0x92a4c9, 0x232f48))?;
                stmt.execute(("dark", 0x1152d4, 0x0a0a0a, 0x92a4c9, 0x232f48))?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn get_all_from_db(
        database: &Database,
    ) -> Result<HashMap<String, Self>, Box<dyn Error>> {
        let mut themes = database
            .pool
            .conn(|conn| {
                let mut stmt = conn.prepare(
                    r"
                        SELECT
                            name, primary_color, background_color, text_color, border_color
                        FROM themes
                    ",
                )?;
                let mut rows = stmt.query([])?;
                let mut themes: HashMap<String, Theme> = HashMap::new();
                while let Some(row) = rows.next()? {
                    let name = row.get(0)?;
                    let primary: u32 = row.get(1)?;
                    let background: u32 = row.get(2)?;
                    let text: u32 = row.get(3)?;
                    let border: u32 = row.get(4)?;
                    themes.insert(
                        name,
                        Theme::new(
                            Color::from_u32(primary),
                            Color::from_u32(background),
                            Color::from_u32(text),
                            Color::from_u32(border),
                        ),
                    );
                }
                Ok(themes)
            })
            .await?;
        themes.insert(String::from("Light"), LIGHT_THEME);
        themes.insert(String::from("Dark"), DARK_THEME);
        Ok(themes)
    }

    pub fn get_all() -> HashMap<String, Self> {
        let mut themes = HashMap::new();
        themes.insert(String::from("Light"), LIGHT_THEME);
        themes.insert(String::from("Dark"), DARK_THEME);
        themes
    }

    pub fn get_user_themes() -> Result<HashMap<String, Self>, Box<dyn Error>> {
        let file = File::open(USER_THEMES_FILE)?;
        let reader = BufReader::new(file);
        let themes: HashMap<String, Theme> = serde_json::from_reader(reader)?;

        for (key, theme) in &themes {
            println!("{}: {:?}", key, theme);
        }

        Ok(themes)
    }
}
