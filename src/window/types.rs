use crate::state::{Position, Size};
use dioxus_desktop::tao::dpi::{LogicalPosition, LogicalSize};

#[derive(Clone, Debug, Default)]
pub struct WindowMetrics {
    pub position: Position,
    pub size: Size,
}

#[derive(Clone, Copy, Debug)]
pub struct ResolvedWindowValue {
    pub position: LogicalPosition<f64>,
    pub size: LogicalSize<f64>,
}
