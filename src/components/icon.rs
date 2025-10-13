use dioxus::prelude::*;
use std::fmt;

const TABLER_SPRITE: Asset = asset!("/assets/dist/icons/tabler-sprite.svg");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconName {
    Sun,
    Moon,
    Contrast2,
    ChevronLeft,
    ChevronRight,
    File,
    FolderOpen,
    Command,
    Click,
}

impl fmt::Display for IconName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            IconName::Sun => "sun",
            IconName::Moon => "moon",
            IconName::Contrast2 => "contrast-2",
            IconName::ChevronLeft => "chevron-left",
            IconName::ChevronRight => "chevron-right",
            IconName::File => "file",
            IconName::FolderOpen => "folder-open",
            IconName::Command => "command",
            IconName::Click => "click",
        };
        write!(f, "{}", name)
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    pub name: IconName,
    #[props(default = 20)]
    pub size: u32,
    #[props(default = "")]
    pub class: &'static str,
}

#[component]
pub fn Icon(props: IconProps) -> Element {
    let sprite_url = TABLER_SPRITE.to_string();
    let icon_id = format!("tabler-{}", props.name);

    rsx! {
        svg {
            class: "icon {props.class}",
            width: "{props.size}",
            height: "{props.size}",
            "aria-hidden": "true",
            r#use {
                href: "{sprite_url}#{icon_id}"
            }
        }
    }
}
