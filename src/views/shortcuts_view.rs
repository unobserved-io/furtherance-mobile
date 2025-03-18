// Furtherance - Track your time without being tracked
// Copyright (C) 2025  Ricky Kresslein <rk@unobserved.io>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use dioxus::prelude::*;
use palette::{color_difference::Wcag21RelativeContrast, Srgb};

use crate::{
    constants::SHORTCUTS_CSS,
    helpers::{actions, color_utils::FromHex},
    models::fur_shortcut::FurShortcut,
    state,
};

#[component]
pub fn ShortcutsView() -> Element {
    rsx! {
        document::Stylesheet { href: SHORTCUTS_CSS }

        div { id: "shortcuts",
            for shortcut in use_context::<state::AllShortcuts>().shortcuts.read().iter() {
                ShortcutItem { shortcut: shortcut.clone() }
            }
        }
    }
}

#[component]
pub fn ShortcutItem(shortcut: FurShortcut) -> Element {
    let styled_rate = format!("${:.2}", shortcut.rate);
    let bg_color = format!("background-color: {};", shortcut.color_hex);
    let bg_srgb = match Srgb::from_hex(&shortcut.color_hex) {
        Ok(color) => color,
        Err(_) => Srgb::new(0.694, 0.475, 0.945),
    };
    let text_color = if is_dark_color(bg_srgb) {
        "color: white;"
    } else {
        "color: black;"
    };

    rsx! {
        button {
            class: "shortcut-bubble",
            style: bg_color,
            onclick: move |_| {
                actions::start_timer_with_task(shortcut.to_string());
            },
            div { class: "shortcut-text", style: text_color,
                p { class: "bold", "{shortcut.name}" }
                if !shortcut.project.is_empty() {
                    p { class: "shortcut-details", "@{shortcut.project}" }
                }
                if !shortcut.tags.is_empty() {
                    p { class: "shortcut-details", "#{shortcut.tags}" }
                }
                if shortcut.rate > 0.0 {
                    p { class: "shortcut-details", "${styled_rate}" }
                }
            }
        }
    }
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.5
}
