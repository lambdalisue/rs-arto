use dioxus::desktop::tao::window::WindowId;
use dioxus::desktop::{window, Config, WeakDesktopContext, WindowBuilder};
use dioxus::prelude::*;

use std::cell::RefCell;
use std::collections::HashMap;

use crate::assets::MAIN_STYLE;
use crate::components::mermaid_window::{generate_diagram_id, MermaidWindow, MermaidWindowProps};
use crate::theme::Theme;

use super::index::build_mermaid_window_index;
use super::main::get_last_focused_window;

struct ChildWindowEntry {
    handle: WeakDesktopContext,
    window_id: WindowId,
    parent_id: WindowId,
}

impl ChildWindowEntry {
    fn is_alive(&self) -> bool {
        self.handle.upgrade().is_some()
    }

    fn focus(&self) -> bool {
        self.handle.upgrade().is_some_and(|ctx| {
            ctx.window.set_focus();
            true
        })
    }

    fn close(&self) {
        if let Some(ctx) = self.handle.upgrade() {
            ctx.close();
        }
    }

    fn is_window(&self, window_id: WindowId) -> bool {
        self.window_id == window_id
    }

    fn is_child_of(&self, parent_id: WindowId) -> bool {
        self.parent_id == parent_id
    }
}

enum ChildWindowState {
    Pending { parent_id: WindowId },
    Created(ChildWindowEntry),
}

thread_local! {
    static CHILD_WINDOWS: RefCell<HashMap<String, ChildWindowState>> = RefCell::new(HashMap::new());
}

pub(crate) fn resolve_to_parent_window(window_id: WindowId) -> WindowId {
    CHILD_WINDOWS.with(|windows| {
        windows
            .borrow()
            .values()
            .find_map(|state| match state {
                ChildWindowState::Created(entry) if entry.is_window(window_id) => {
                    Some(entry.parent_id)
                }
                _ => None,
            })
            .unwrap_or(window_id)
    })
}

pub fn close_child_windows_for_parent(parent_id: WindowId) {
    CHILD_WINDOWS.with(|windows| {
        windows.borrow_mut().retain(|_, state| match state {
            ChildWindowState::Pending {
                parent_id: pending_parent,
            } => *pending_parent != parent_id,
            ChildWindowState::Created(entry) => {
                if entry.is_child_of(parent_id) {
                    entry.close();
                    false
                } else {
                    entry.is_alive()
                }
            }
        });
    });
}

pub fn close_child_windows_for_last_focused() {
    if let Some(window_id) = get_last_focused_window() {
        let parent_id = resolve_to_parent_window(window_id);
        close_child_windows_for_parent(parent_id)
    }
}

pub fn open_or_focus_mermaid_window(source: String, theme: Theme) {
    let diagram_id = generate_diagram_id(&source);
    let parent_id = window().id();

    // Check if window already exists and can be focused
    let needs_creation = CHILD_WINDOWS.with(|windows| {
        let mut windows = windows.borrow_mut();
        windows.retain(|_, state| match state {
            ChildWindowState::Pending { .. } => true,
            ChildWindowState::Created(entry) => entry.is_alive(),
        });

        match windows.get(&diagram_id) {
            Some(ChildWindowState::Created(entry)) => !entry.focus(),
            Some(ChildWindowState::Pending { .. }) => false,
            None => {
                windows.insert(diagram_id.clone(), ChildWindowState::Pending { parent_id });
                true
            }
        }
    });

    if needs_creation {
        dioxus_core::spawn(create_and_register_mermaid_window(
            source, diagram_id, theme, parent_id,
        ));
    }
}

async fn create_and_register_mermaid_window(
    source: String,
    diagram_id: String,
    theme: Theme,
    parent_id: WindowId,
) {
    let dom = VirtualDom::new_with_props(
        MermaidWindow,
        MermaidWindowProps {
            source,
            diagram_id: diagram_id.clone(),
            theme,
        },
    );

    let config = Config::new()
        .with_menu(None)
        .with_window(WindowBuilder::new().with_title("Mermaid Viewer"))
        .with_custom_head(indoc::formatdoc! {r#"<link rel="stylesheet" href="{MAIN_STYLE}">"#})
        .with_custom_index(build_mermaid_window_index(theme));

    let pending = window().new_window(dom, config);
    let ctx = pending.await;
    let weak_handle = std::rc::Rc::downgrade(&ctx);
    let window_id = ctx.window.id();

    CHILD_WINDOWS.with(|windows| {
        windows.borrow_mut().insert(
            diagram_id,
            ChildWindowState::Created(ChildWindowEntry {
                handle: weak_handle,
                window_id,
                parent_id,
            }),
        );
    });
}
