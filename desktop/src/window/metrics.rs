use dioxus::desktop::tao::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::tao::window::Window;
use std::sync::OnceLock;

use crate::state::{Position, Size};

use super::types::WindowMetrics;

struct OuterToInnerMetrics {
    width_delta: u32,
    height_delta: u32,
}

static OUTER_TO_INNER_METRICS: OnceLock<OuterToInnerMetrics> = OnceLock::new();

pub fn update_outer_to_inner_metrics(window: &Window) {
    let scale = window.scale_factor();
    let outer_size = window.outer_size().to_logical::<u32>(scale);
    let inner_size = window.inner_size().to_logical::<u32>(scale);
    OUTER_TO_INNER_METRICS.get_or_init(|| OuterToInnerMetrics {
        width_delta: outer_size.width.saturating_sub(inner_size.width),
        height_delta: outer_size.height.saturating_sub(inner_size.height),
    });
}

pub fn outer_to_inner_size(outer: LogicalSize<u32>) -> LogicalSize<u32> {
    if let Some(metrics) = OUTER_TO_INNER_METRICS.get() {
        LogicalSize::new(
            outer.width.saturating_sub(metrics.width_delta),
            outer.height.saturating_sub(metrics.height_delta),
        )
    } else {
        outer
    }
}

pub fn capture_window_metrics(window: &Window) -> WindowMetrics {
    let scale = window.scale_factor();
    let position = window
        .outer_position()
        .map(|pos| pos.to_logical::<i32>(scale))
        .unwrap_or_else(|_| LogicalPosition::new(0, 0));
    let outer_size = window.outer_size().to_logical::<u32>(scale);
    let inner_size = if OUTER_TO_INNER_METRICS.get().is_some() {
        outer_to_inner_size(outer_size)
    } else {
        window.inner_size().to_logical::<u32>(scale)
    };
    WindowMetrics {
        position: Position {
            x: position.x,
            y: position.y,
        },
        size: Size {
            width: inner_size.width,
            height: inner_size.height,
        },
    }
}
