use crate::state::{Position, Size};

#[derive(Clone, Debug, Default)]
pub struct WindowMetrics {
    pub position: Position,
    pub size: Size,
}
