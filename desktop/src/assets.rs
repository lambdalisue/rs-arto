use dioxus::prelude::*;

pub const MAIN_SCRIPT: Asset = asset!("/assets/dist/main.js");
pub const MAIN_STYLE: Asset = asset!("/assets/dist/main.css");

// Embed header image as base64 at compile time
const HEADER_IMAGE_BYTES: &[u8] = include_bytes!("../assets/arto-header-welcome.png");

// Generate data URL for the header image
fn generate_header_data_url() -> String {
    use base64::{engine::general_purpose, Engine as _};
    let base64_data = general_purpose::STANDARD.encode(HEADER_IMAGE_BYTES);
    format!("data:image/png;base64,{}", base64_data)
}

// Embed and process default markdown content at runtime
pub fn get_default_markdown_content() -> String {
    let template = include_str!("../assets/welcome.md");
    let header_data_url = generate_header_data_url();

    // Replace relative image path with data URL
    template.replace("../assets/arto-header-welcome.png", &header_data_url)
}
