# TRSS – Terminal RSS Reader

TRSS is a fast, lightweight **Terminal-based RSS reader** written in **Rust**, built using the **Ratatui** library.  
It allows users to add RSS feed URLs, store them locally using **SQLite**, and read feed posts directly inside the terminal.

---

## 🚀 Features

- 📡 Add RSS feeds using feed URLs
- 📰 View and browse RSS posts in a TUI (Terminal User Interface)
- 💾 Persistent local storage using SQLite
- 🖥️ Keyboard-driven terminal interface powered by Ratatui
- 🔄 Refresh feeds on demand *(optional / extendable)*

---

## 📦 Installation

```bash
git clone https://github.com/ShUbHaM13M/trss.git
cd trss

cargo build --release

cargo run
# or use the release binary
./target/release/trss
```

---

## Project Structure

trss/
├── src/
│   ├── main.rs
│   ├── models/
│   ├── screens/
│   ├── utils/
│   ├── app.rs/
│   ├── events.rs/
│   └── widgets/
├── Cargo.toml
└── README.md

---

## 📈 Future Improvements

- **Enhanced User Interface**: Improve the visual design and layout of the TUI.
- **Customizable Settings**: Allow users to customize settings such as feed refresh intervals and background sync.
- **Theme Customization**: Enable users to choose from different color schemes or create their own.
- **Feed Categories and tags**: Organize feeds into categories and apply tags for better organization.
- **Customizable Keybindings**: Customize keyboard shortcuts for navigation and actions.

--- 

## 🤝 Contributing

Contributions are welcome!
1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Open a pull request
