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
use dioxus_free_icons::{icons::bs_icons::BsPlus, Icon};
use palette::{color_difference::Wcag21RelativeContrast, Srgb};
use rand::Rng;

use crate::{
    constants::{SHEET_CSS, SHORTCUTS_CSS},
    database,
    helpers::{
        actions,
        color_utils::FromHex,
        formatters,
        views::{shortcuts::update_all_shortcuts, task_input::validate_task_input},
    },
    loc,
    localization::Localization,
    models::fur_shortcut::FurShortcut,
    state,
};

#[component]
pub fn ShortcutsView() -> Element {
    let sheets = use_context::<state::FurState>().sheets.read().clone();

    rsx! {
        document::Stylesheet { href: SHORTCUTS_CSS }
        document::Stylesheet { href: SHEET_CSS }

        AddNewShortcut {}

        div { id: "shortcuts",
            for shortcut in use_context::<state::FurState>().shortcuts.read().iter() {
                ShortcutItem { shortcut: shortcut.clone() }
            }
        }

        div { class: if sheets.new_shortcut_is_shown { "overlay visible" } else { "overlay" },
            ""
        }
        div { class: if sheets.new_shortcut_is_shown { "sheet visible" } else { "sheet" }, NewShortcutSheet {} }
    }
}

#[component]
pub fn AddNewShortcut() -> Element {
    rsx! {
        div { id: "add-new-shortcut",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_shortcut_is_shown = true;
                    state.sheets.set(new_sheets);
                },
                Icon { icon: BsPlus, width: 40, height: 40 }
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
                    p { class: "shortcut-details", "{styled_rate}" }
                }
            }
        }
    }
}

#[component]
fn NewShortcutSheet() -> Element {
    let mut task_input = use_signal(|| String::new());
    let mut color_hex = use_signal(|| random_color());
    let save_text = loc!("save");
    let cancel_text = loc!("cancel");
    let color_text = loc!("color");
    let new_shortcut_text = loc!("new-shortcut");

    rsx! {
        div { class: "sheet-contents",
            h2 { "{new_shortcut_text}" }
            input {
                class: "sheet-task-input",
                value: "{task_input}",
                oninput: move |event| {
                    let new_value = validate_task_input(event.value());
                    task_input.set(new_value);
                },
                placeholder: loc!("task-input-placeholder"),
            }

            div { class: "color-selector",
                br {}
                label { class: "sheet-label", "{color_text}" }
                input {
                    r#type: "color",
                    value: "{color_hex}",
                    oninput: move |event| { color_hex.set(event.value()) },
                }
            }

            br {}
            button {
                class: "sheet-cancel-button",
                onclick: move |_| {
                    task_input.set(String::new());
                    color_hex.set(random_color());
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_shortcut_is_shown = false;
                    state.sheets.set(new_sheets);
                },
                "{cancel_text}"
            }
            button {
                class: "sheet-primary-button",
                onclick: move |event| {
                    if task_input.read().trim().is_empty() {
                        event.prevent_default();
                    } else {
                        let (name, project, tags, rate) = formatters::split_task_input(
                            &task_input.cloned(),
                        );
                        database::shortcuts::insert_shortcut(
                                &FurShortcut::new(
                                    name,
                                    tags,
                                    project,
                                    rate,
                                    String::new(),
                                    color_hex.cloned(),
                                ),
                            )
                            .expect("Couldn't write task to database.");
                        task_input.set(String::new());
                        color_hex.set(random_color());
                        let mut state = use_context::<state::FurState>();
                        let mut new_sheets = state.sheets.read().clone();
                        new_sheets.new_shortcut_is_shown = false;
                        state.sheets.set(new_sheets);
                        update_all_shortcuts();
                    }
                },
                "{save_text}"
            }
        }
    }
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.5
}

fn random_color() -> String {
    let mut rng = rand::thread_rng();
    format!("#{:06x}", rng.gen::<u32>() & 0xFFFFFF)
}
