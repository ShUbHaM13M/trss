use ratatui::widgets::WidgetRef;

pub mod feed_list;
pub mod sidebar;

pub trait Focusable: WidgetRef {
    fn focus(&mut self);
    fn blur(&mut self);
}

pub trait WidgetExt<T>: Focusable {
    fn update(&mut self, state: T);
    fn handle_input(&mut self, event: &crate::event::Event);
}
