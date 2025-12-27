use crate::components::icon::{Icon, IconName};
use dioxus::prelude::*;

const ARTO_ICON: Asset = asset!("/assets/arto-app.png");

#[component]
pub fn AboutTab() -> Element {
    let version_text = format!("Version {}", env!("CARGO_PKG_VERSION"));

    rsx! {
        div {
            class: "about-page",

            div {
                class: "about-container",

                // Icon
                div {
                    class: "about-icon",
                    img {
                        src: "{ARTO_ICON}",
                        alt: "Arto",
                    }
                }

                // Title
                h2 { class: "about-title", "Arto" }

                // Version
                p { class: "about-version", "{version_text}" }

                // Tagline
                p { class: "about-tagline", "The Art of Reading Markdown." }

                // Description
                p { class: "about-description",
                    "A local app that faithfully recreates GitHub-style Markdown rendering for a beautiful reading experience."
                }

                // Links (card style like no-file-hints)
                div {
                    class: "about-links",
                    a {
                        href: "https://github.com/arto-app/Arto",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "about-link",
                        span { class: "about-link-icon", Icon { name: IconName::BrandGithub, size: 20 } }
                        span { class: "about-link-text", "View on GitHub" }
                    }
                    a {
                        href: "https://github.com/arto-app/Arto/issues",
                        target: "_blank",
                        rel: "noopener noreferrer",
                        class: "about-link",
                        span { class: "about-link-icon", Icon { name: IconName::Bug, size: 20 } }
                        span { class: "about-link-text", "Report an Issue" }
                    }
                }

                // Footer
                div {
                    class: "about-footer",
                    p { "Created by lambdalisue" }
                    p { "Copyright © 2025 lambdalisue" }
                }
            }
        }
    }
}
