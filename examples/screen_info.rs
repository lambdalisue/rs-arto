use arto::utils::screen::{get_cursor_display, get_primary_display};
use display_info::DisplayInfo;
use mouse_position::mouse_position::Mouse;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn format_display(display: Option<DisplayInfo>) -> String {
    match display {
        Some(display) => format!(
            "x={} y={} width={} height={} scale={} primary={}",
            display.x,
            display.y,
            display.width,
            display.height,
            display.scale_factor,
            display.is_primary
        ),
        None => "None".to_string(),
    }
}

fn format_mouse_position() -> String {
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => format!("x={} y={}", x, y),
        Mouse::Error => "Error".to_string(),
    }
}

fn main() {
    loop {
        let primary = get_primary_display();
        let cursor = get_cursor_display();
        let mouse = format_mouse_position();

        print!("\x1B[2J\x1B[H");
        println!("Primary display: {}", format_display(primary));
        println!("Cursor display:  {}", format_display(cursor));
        println!("Mouse position:  {}", mouse);
        io::stdout().flush().ok();

        thread::sleep(Duration::from_millis(200));
    }
}
