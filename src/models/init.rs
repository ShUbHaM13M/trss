pub const SCHEMA_QUERIES: &[&str] = &[
    r#"
    CREATE TABLE IF NOT EXISTS feeds (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        subtitle TEXT,
        url TEXT NOT NULL UNIQUE,
        image TEXT,
        last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        active INTEGER DEFAULT (1)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS feed_items (
        id TEXT PRIMARY KEY,
        feed_id INTEGER NOT NULL,
        link TEXT NOT NULL,
        title TEXT NOT NULL,
        summary TEXT,
        content TEXT,
        thumbnail TEXT,
        published TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS favourite_feeds (
        feed_item_id TEXT PRIMARY KEY,
        FOREIGN KEY (feed_item_id) REFERENCES feed_items(id)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS readlist (
        feed_item_id TEXT PRIMARY KEY,
        FOREIGN KEY (feed_item_id) REFERENCES feed_items(id)
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS settings (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        value TEXT NOT NULL
    );
    "#,
    r#"
    CREATE TABLE IF NOT EXISTS themes (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT UNIQUE NOT NULL,
        primary_color INTEGER NOT NULL,
        background_color INTEGER NOT NULL,
        text_color INTEGER NOT NULL,
        border_color INTEGER NOT NULL
    );
    "#,
];
