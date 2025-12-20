#[macro_use]
extern crate lazy_static;
use crate::app::App;
use crate::utils::parse_html::parse_html;

pub mod app;
pub mod event;
pub mod models;
pub mod screens;
pub mod utils;
pub mod widgets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if true {
        let mut terminal = ratatui::init();
        let mut app = App::new().await;
        terminal.clear()?;
        let result = app.run(terminal);
        ratatui::restore();
        result
    } else {
        let html = r#"
<pre>
    <code class="language-c">
        ~filename module.h
        #ifndef MODULE_H_
        #define MODULE_H_
            void (*module_init)(void) = NULL;
                void *(*module_pre_reload)(void) = NULL;
                    void (*module_post_reload)(void *) = NULL;
                void (*module_update)(void) = NULL; #endif
        // MODULE_H_

        int main (void) {
            printf("Hello World\n");
            return 0;
        }
    </code>
</pre>
"#;
        let html = parse_html(html.to_string());
        println!("{:?}", html);
        Ok(())
    }
}
