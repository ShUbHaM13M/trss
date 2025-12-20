use ratatui::widgets::WidgetRef;

pub mod feed_list;
pub mod sidebar;

pub trait Focusable: WidgetRef {
    fn focus(&mut self);
    fn blur(&mut self);
    fn get_child_focused_index(&self) -> Option<usize>;
    fn set_child_focused_index(&mut self, index: Option<usize>);
}

pub trait WidgetExt<T>: Focusable {
    fn update(&mut self, state: T);
    fn handle_input(&mut self, state: &T, event: &crate::event::Event) -> Option<T>;
    fn get_child_widgets(&self) -> Option<Vec<String>>;
}
