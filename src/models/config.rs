use std::{collections::HashMap, fs::File, io::BufReader};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::Deserialize;

use super::theme::Theme;

pub const CONFIG_FILE: &str = "trss_config.json";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub themes: HashMap<String, Theme>,
    pub keymap: Keymap,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Keymap {
    pub home: HashMap<String, Vec<String>>,
    pub article: HashMap<String, Vec<String>>,
}

impl UserConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(serde_json::from_reader(BufReader::new(File::open(
            CONFIG_FILE,
        )?))?)
    }
}

impl Keymap {
    pub fn matches(&self, context: &str, action: &str, key: KeyEvent) -> bool {
        let custom = match context {
            "home" => self.home.get(action),
            "article" => self.article.get(action),
            _ => None,
        };
        if let Some(keys) = custom {
            return keys.iter().any(|binding| binding_matches(binding, key));
        }

        let defaults: &[&str] = match (context, action) {
            ("home", "quit") => &["q", "Esc"],
            ("home", "down") => &["j", "Down"],
            ("home", "up") => &["k", "Up"],
            ("home", "page_down") => &["PageDown", "Ctrl+d"],
            ("home", "page_up") => &["PageUp", "Ctrl+u"],
            ("home", "top") => &["Home"],
            ("home", "bottom") => &["G"],
            ("home", "open") => &["Enter"],
            ("home", "search") => &["/"],
            ("home", "add_feed") => &["a"],
            ("home", "refresh") => &["r"],
            ("home", "delete_feed") => &["d"],
            ("home", "toggle_sidebar") => &["Ctrl+b"],
            ("article", "back") => &["Esc", "Backspace", "Left", "h", "s"],
            ("article", "down") => &["j", "Down"],
            ("article", "up") => &["k", "Up"],
            ("article", "page_down") => &["PageDown", "Space"],
            ("article", "page_up") => &["PageUp"],
            ("article", "top") => &["Home", "g"],
            ("article", "bottom") => &["End", "G"],
            ("article", "next") => &["n"],
            ("article", "previous") => &["p"],
            ("article", "favourite") => &["F1", "f"],
            ("article", "readlist") => &["r"],
            _ => &[],
        };
        defaults.iter().any(|binding| binding_matches(binding, key))
    }
}

fn binding_matches(binding: &str, key: KeyEvent) -> bool {
    let mut parts = binding.split('+').map(str::trim).collect::<Vec<_>>();
    let Some(code) = parts.pop() else {
        return false;
    };
    let mut modifiers = KeyModifiers::NONE;
    for modifier in parts {
        match modifier.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
            "alt" => modifiers |= KeyModifiers::ALT,
            "shift" => modifiers |= KeyModifiers::SHIFT,
            _ => return false,
        }
    }
    if key.modifiers != modifiers {
        return false;
    }
    match code.to_ascii_lowercase().as_str() {
        "esc" | "escape" => key.code == KeyCode::Esc,
        "enter" | "return" => key.code == KeyCode::Enter,
        "backspace" => key.code == KeyCode::Backspace,
        "tab" => key.code == KeyCode::Tab,
        "backtab" => key.code == KeyCode::BackTab,
        "up" => key.code == KeyCode::Up,
        "down" => key.code == KeyCode::Down,
        "left" => key.code == KeyCode::Left,
        "right" => key.code == KeyCode::Right,
        "home" => key.code == KeyCode::Home,
        "end" => key.code == KeyCode::End,
        "pageup" => key.code == KeyCode::PageUp,
        "pagedown" => key.code == KeyCode::PageDown,
        "space" => key.code == KeyCode::Char(' '),
        "f1" => key.code == KeyCode::F(1),
        _ => {
            let mut chars = code.chars();
            matches!((chars.next(), chars.next()), (Some(ch), None) if key.code == KeyCode::Char(ch))
        }
    }
}
