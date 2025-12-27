pub mod child;
pub mod index;
pub mod main;
pub mod metrics;
pub mod settings;
mod types;

pub use child::{
    close_child_windows_for_last_focused, close_child_windows_for_parent,
    open_or_focus_mermaid_window,
};
pub use main::{
    close_all_main_windows, create_main_window_config, create_new_main_window_with_empty,
    create_new_main_window_with_file, focus_last_focused_main_window, has_any_main_windows,
    register_main_window, update_last_focused_window, CreateMainWindowConfigParams,
};
