use dioxus::desktop::tao::dpi::LogicalSize;
use display_info::DisplayInfo;
use mouse_position::mouse_position::Mouse;

pub fn get_current_display_size() -> Option<LogicalSize<f64>> {
    get_cursor_display()
        .or_else(get_primary_display)
        .map(to_logical_size)
}

pub fn get_primary_display() -> Option<DisplayInfo> {
    let displays = DisplayInfo::all().ok()?;
    displays
        .iter()
        .find(|display| display.is_primary)
        .copied()
        .or_else(|| displays.first().copied())
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
        .or_else(|| {
            let fx = x as f64;
            let fy = y as f64;
            displays.iter().find(|display| {
                let scale = display.scale_factor as f64;
                if scale <= 0.0 {
                    return false;
                }
                let left = display.x as f64 / scale;
                let top = display.y as f64 / scale;
                let right = left + display.width as f64 / scale;
                let bottom = top + display.height as f64 / scale;
                fx >= left && fx < right && fy >= top && fy < bottom
            })
        })
        .copied()
        .or_else(|| displays.first().copied())
}

fn to_logical_size(di: DisplayInfo) -> LogicalSize<f64> {
    let scale = di.scale_factor as f64;
    LogicalSize::new((di.width as f64) / scale, (di.height as f64) / scale)
}
