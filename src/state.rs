// State module - manages application state

mod app_state;
pub use app_state::{AppState, Tab, TabContent};

mod persistence;
pub use persistence::{PersistedState, LAST_FOCUSED_STATE};
