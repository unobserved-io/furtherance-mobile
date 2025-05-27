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
use rand::Rng;

use crate::{
    constants::{SHEET_CSS, SHORTCUTS_CSS},
    database,
    helpers::{
        actions,
        color_utils::FromHex,
        formatters,
        icons::{BsPencil, BsPencilFill, BsPlusLg, BsTrash3, Icon},
        server::sync::sync_after_change,
        views::{shortcuts::update_all_shortcuts, task_input::validate_task_input},
    },
    loc,
    localization::Localization,
    models::fur_shortcut::FurShortcut,
    state::{self, SHORTCUT_ID_TO_DELETE},
};

#[component]
pub fn ShortcutsView() -> Element {
    let sheets = state::SHEETS.cloned();
    let edit_mode = use_signal(|| false);

    rsx! {
        document::Stylesheet { href: SHORTCUTS_CSS }
        document::Stylesheet { href: SHEET_CSS }

        TopButtons { edit_mode }

        div { id: "shortcuts",
            for shortcut in state::SHORTCUTS.read().iter() {
                ShortcutItem { shortcut: shortcut.clone(), edit_mode }
            }
        }

        div { class: if sheets.new_shortcut_is_shown || sheets.edit_shortcut_sheet.is_some() { "overlay visible" } else { "overlay" },
            ""
        }
        div { class: if sheets.new_shortcut_is_shown { "sheet visible" } else { "sheet" }, NewShortcutSheet {} }
        div { class: if sheets.edit_shortcut_sheet.is_some() { "sheet visible" } else { "sheet" },
            if sheets.edit_shortcut_sheet.is_some() {
                EditShortcutSheet { shortcut: sheets.edit_shortcut_sheet.clone() }
            }
        }
    }
}

#[component]
pub fn TopButtons(edit_mode: Signal<bool>) -> Element {
    rsx! {
        div { class: "top-shortcut-buttons",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    edit_mode.set(!edit_mode.cloned());
                },
                if edit_mode.cloned() {
                    Icon { icon: BsPencilFill, width: 30, height: 30 }
                } else {
                    Icon { icon: BsPencil, width: 30, height: 30 }
                }
            }
            if !edit_mode.cloned() {
                button {
                    class: "no-bg-button",
                    onclick: move |_| {
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.new_shortcut_is_shown = true;
                        *state::SHEETS.write() = new_sheets;
                    },
                    Icon { icon: BsPlusLg, width: 30, height: 30 }
                }
            }
        }
    }
}

#[component]
pub fn ShortcutItem(shortcut: FurShortcut, edit_mode: Signal<bool>) -> Element {
    let shortcut_clone = shortcut.clone();
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
            class: if edit_mode.cloned() && state::SHEETS.read().edit_shortcut_sheet.is_none() { "shortcut-bubble wiggle" } else { "shortcut-bubble" },
            style: bg_color,
            onclick: move |_| {
                if edit_mode.cloned() {
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.edit_shortcut_sheet = Some(shortcut_clone.clone());
                    *state::SHEETS.write() = new_sheets;
                } else {
                    actions::start_timer_with_task(shortcut.to_string())
                };
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
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.new_shortcut_is_shown = false;
                    *state::SHEETS.write() = new_sheets;
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
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.new_shortcut_is_shown = false;
                        *state::SHEETS.write() = new_sheets;
                        update_all_shortcuts();
                        sync_after_change();
                    }
                },
                "{save_text}"
            }
        }
    }
}

#[component]
fn EditShortcutSheet(shortcut: Option<FurShortcut>) -> Element {
    if let Some(shortcut) = shortcut {
        let mut shortcut_clone = shortcut.clone();
        let shortcut_clone_two = shortcut.clone();
        let mut task_input = use_signal(|| shortcut.to_string());
        let mut color_hex = use_signal(|| shortcut.color_hex);

        rsx! {
            div { class: "sheet-contents",

                div { id: "group-buttons-row",
                    div {
                        button {
                            class: "no-bg-button",
                            onclick: move |_| {
                                fn delete_shortcut() {
                                    if let Some(shortcut_id) = SHORTCUT_ID_TO_DELETE.cloned() {
                                        if let Err(e) = database::shortcuts::delete_shortcut_by_id(
                                            &shortcut_id,
                                        ) {
                                            eprintln!("Failed to delete shortcut: {}", e);
                                        }
                                    }
                                    let mut alert = state::ALERT.cloned();
                                    let mut new_sheets = state::SHEETS.cloned();
                                    new_sheets.edit_shortcut_sheet = None;
                                    *state::SHEETS.write() = new_sheets;
                                    *SHORTCUT_ID_TO_DELETE.write() = None;
                                    alert.close();
                                    *state::ALERT.write() = alert.clone();
                                    update_all_shortcuts();
                                    sync_after_change();
                                }
                                fn close_alert() {
                                    let mut alert = state::ALERT.cloned();
                                    *SHORTCUT_ID_TO_DELETE.write() = None;
                                    alert.close();
                                    *state::ALERT.write() = alert.clone();
                                }
                                let settings = state::SETTINGS.cloned();
                                let mut alert = state::ALERT.cloned();
                                if settings.show_delete_confirmation {
                                    *SHORTCUT_ID_TO_DELETE.write() = Some(shortcut_clone_two.uid.clone());
                                    alert.is_shown = true;
                                    alert.title = loc!("delete-shortcut-question");
                                    alert.message = loc!("delete-shortcut-description");
                                    alert.confirm_button = (loc!("delete"), || delete_shortcut());
                                    alert.cancel_button = Some((loc!("cancel"), || close_alert()));
                                    *state::ALERT.write() = alert.clone();
                                } else {
                                    if let Err(e) = database::shortcuts::delete_shortcut_by_id(
                                        &shortcut_clone_two.uid,
                                    ) {
                                        eprintln!("Failed to delete shortcut: {}", e);
                                    }
                                    let mut new_sheets = state::SHEETS.cloned();
                                    new_sheets.edit_shortcut_sheet = None;
                                    *state::SHEETS.write() = new_sheets;
                                    update_all_shortcuts();
                                    sync_after_change();
                                }
                            },
                            Icon { icon: BsTrash3, width: 25, height: 25 }
                        }
                    }
                }


                h2 { {loc!("edit-shortcut")} }
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
                    label { class: "sheet-label", {loc!("color")} }
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
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.edit_shortcut_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                    },
                    {loc!("cancel")}
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
                            shortcut_clone.name = name.clone();
                            shortcut_clone.project = project.clone();
                            shortcut_clone.tags = tags.clone();
                            shortcut_clone.rate = rate.clone();
                            shortcut_clone.color_hex = color_hex.cloned();
                            database::shortcuts::update_shortcut(&shortcut_clone)
                                .expect("Couldn't write task to database.");
                            let mut new_sheets = state::SHEETS.cloned();
                            new_sheets.edit_shortcut_sheet = None;
                            *state::SHEETS.write() = new_sheets;
                            update_all_shortcuts();
                            sync_after_change();
                        }
                    },
                    {loc!("save")}
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn is_dark_color(color: Srgb) -> bool {
    color.relative_luminance().luma < 0.5
}

fn random_color() -> String {
    let mut rng = rand::thread_rng();
    format!("#{:06x}", rng.gen::<u32>() & 0xFFFFFF)
}
