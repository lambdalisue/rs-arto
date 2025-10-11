use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/dist/main.css");
const MAIN_SCRIPT: Asset = asset!("/assets/dist/main.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Script { r#type: "module", src: MAIN_SCRIPT }
    }
}
