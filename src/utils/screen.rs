use dioxus::desktop::tao::dpi::LogicalSize;
use display_info::DisplayInfo;
use mouse_position::mouse_position::Mouse;

pub fn get_current_display_size() -> Option<LogicalSize<u32>> {
    get_cursor_display()
        .or_else(get_primary_display)
        .map(to_logical_size)
}

pub fn get_primary_display() -> Option<DisplayInfo> {
    let displays = DisplayInfo::all().ok()?;
    displays
        .iter()
        .find(|display| display.is_primary)
        .cloned()
        .or_else(|| displays.first().cloned())
}

pub fn get_cursor_display() -> Option<DisplayInfo> {
    let (x, y) = match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        Mouse::Error => return None,
    };

    if let Ok(display) = DisplayInfo::from_point(x, y) {
        return Some(display);
    }

    let displays = DisplayInfo::all().ok()?;
    if let Some(primary) = displays.iter().find(|display| display.is_primary) {
        // Some platforms report cursor Y in an inverted coordinate space; try a flipped Y fallback.
        let flipped_y = primary.y + primary.height as i32 - y;
        if let Ok(display) = DisplayInfo::from_point(x, flipped_y) {
            return Some(display);
        }
    }

    displays
        .iter()
        .find(|display| {
            let left = display.x;
            let top = display.y;
            let right = left + display.width as i32;
            let bottom = top + display.height as i32;
            x >= left && x < right && y >= top && y < bottom
        })
        .cloned()
        .or_else(|| displays.first().cloned())
}

fn to_logical_size_from_parts(width: u32, height: u32, scale: f64) -> LogicalSize<u32> {
    let width = (width as f64 / scale).round().max(1.0) as u32;
    let height = (height as f64 / scale).round().max(1.0) as u32;
    LogicalSize::new(width, height)
}

fn to_logical_size(di: DisplayInfo) -> LogicalSize<u32> {
    to_logical_size_from_parts(di.width, di.height, di.scale_factor as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_logical_size_from_parts_scales() {
        let size = to_logical_size_from_parts(100, 50, 2.0);
        assert_eq!(size.width, 50);
        assert_eq!(size.height, 25);
    }

    #[test]
    fn test_to_logical_size_from_parts_minimum() {
        let size = to_logical_size_from_parts(0, 0, 2.0);
        assert_eq!(size.width, 1);
        assert_eq!(size.height, 1);
    }
}
