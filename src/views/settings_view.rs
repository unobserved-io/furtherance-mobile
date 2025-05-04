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

use crate::{
    constants::{OFFICIAL_SERVER, SETTINGS_CSS},
    helpers::{
        server::{self, login::login_button_pressed, logout::logout_button_pressed},
        views::{settings::ServerChoices, timer},
    },
    loc,
    localization::Localization,
    state,
};

#[component]
pub fn SettingsView() -> Element {
    let pomodoro = state::SETTINGS.read().pomodoro;
    let pomodoro_length = state::SETTINGS.read().pomodoro_length;
    let pomodoro_break_length = state::SETTINGS.read().pomodoro_break_length;
    let pomodoro_snooze_length = state::SETTINGS.read().pomodoro_snooze_length;
    let pomodoro_extended_breaks = state::SETTINGS.read().pomodoro_extended_breaks;
    let pomodoro_extended_break_interval = state::SETTINGS.read().pomodoro_extended_break_interval;
    let pomodoro_extended_break_length = state::SETTINGS.read().pomodoro_extended_break_length;

    let show_delete_confirmation = state::SETTINGS.read().show_delete_confirmation;

    let show_task_project = state::SETTINGS.read().show_task_project;
    let show_task_tags = state::SETTINGS.read().show_task_tags;
    let show_task_earnings = state::SETTINGS.read().show_task_earnings;
    let show_seconds = state::SETTINGS.read().show_seconds;
    let show_daily_time_total = state::SETTINGS.read().show_daily_time_total;
    let dynamic_total = state::SETTINGS.read().dynamic_total;
    let days_to_show = state::SETTINGS.read().days_to_show;

    let show_todo_project = state::SETTINGS.read().show_todo_project;
    let show_todo_tags = state::SETTINGS.read().show_todo_tags;
    let show_todo_rate = state::SETTINGS.read().show_todo_rate;

    // sync_server_col = sync_server_col.push(sync_button_row);
    // sync_server_col = sync_server_col.push_maybe(match &self.login_message {
    //     Ok(msg) => {
    //         if msg.is_empty() {
    //             None
    //         } else {
    //             Some(text(msg).style(style::green_text))
    //         }
    //     }
    //     Err(e) => Some(text!("{}", e).style(style::red_text)),
    // });

    rsx! {
        document::Stylesheet { href: SETTINGS_CSS }

        div { id: "settings",
            SettingsTitleRow { title: loc!("sync") }
            div { class: "settings-group",
                SettingsDropDownRow {
                    label: loc!("server"),
                    list_items: ServerChoices::all_as_strings(),
                    selected_item: state::USER_FIELDS.read().server_selection.to_string(),
                    onchange: move |event: Event<FormData>| {
                        let mut user_fields_clone = state::USER_FIELDS.read().clone();
                        if event.data.value() == ServerChoices::Official.to_string() {
                            user_fields_clone.server_selection = ServerChoices::Official;
                            user_fields_clone.server = OFFICIAL_SERVER.to_string();
                        } else {
                            user_fields_clone.server_selection = ServerChoices::Custom;
                            if let Some(fur_user) = state::USER.cloned() {
                                user_fields_clone.server = fur_user.server;
                            } else {
                                user_fields_clone.server = String::new();
                            }
                        }
                        *state::USER_FIELDS.write() = user_fields_clone;
                    },
                }
                if state::USER_FIELDS.read().server_selection == ServerChoices::Custom {
                    SettingsInputRow {
                        label: loc!("server"),
                        input_type: "url".to_string(),
                        value: state::USER_FIELDS.read().server.clone(),
                        placeholder: loc!("server-placeholder"),
                        oninput: move |event: Event<FormData>| {
                            let mut user_fields_clone = state::USER_FIELDS.read().clone();
                            user_fields_clone.server = event.value();
                            *state::USER_FIELDS.write() = user_fields_clone;
                        },
                    }
                }
                SettingsInputRow {
                    label: loc!("email"),
                    input_type: "email".to_string(),
                    value: state::USER_FIELDS.read().email.clone(),
                    placeholder: loc!("email-placeholder"),
                    oninput: move |event: Event<FormData>| {
                        let mut user_fields_clone = state::USER_FIELDS.cloned();
                        user_fields_clone.email = event.value();
                        *state::USER_FIELDS.write() = user_fields_clone;
                    },
                }
                SettingsInputRow {
                    label: loc!("encryption-key"),
                    input_type: "password".to_string(),
                    value: state::USER_FIELDS.read().encryption_key.clone(),
                    placeholder: loc!("key"),
                    oninput: move |event: Event<FormData>| {
                        let mut user_fields_clone = state::USER_FIELDS.cloned();
                        user_fields_clone.encryption_key = event.value();
                        *state::USER_FIELDS.write() = user_fields_clone;
                    },
                }
                if state::USER.read().is_some() {
                    SettingsButtonRow {
                        label: loc!("sync"),
                        dangerous: false,
                        onclick: move |_| {
                            if state::USER.read().is_some()
                                && state::SYNC_MESSAGE
                                    .read()
                                    .iter()
                                    .any(|message| message != &loc!("syncing"))
                            {
                                server::sync::request_sync();
                            }
                        },
                    }
                } else {
                    SettingsButtonRow {
                        label: loc!("sign-up"),
                        dangerous: false,
                        onclick: move |_| {
                            if let Err(e) = webbrowser::open("https://furtherance.app/sync") {
                                eprintln!("Failed to open URL in browser: {}", e);
                            }
                        },
                    }
                }
                SettingsButtonRow {
                    label: if state::USER.read().is_some() { loc!("log-out") } else { loc!("log-in") },
                    dangerous: if state::USER.read().is_some() { true } else { false },
                    onclick: move |_| {
                        if state::USER.read().is_some() {
                            logout_button_pressed();
                        } else {
                            let user_fields = state::USER_FIELDS.cloned();
                            if !user_fields.server.is_empty() && !user_fields.email.is_empty()
                                && !user_fields.encryption_key.is_empty()
                            {
                                login_button_pressed();
                            }
                        }
                    },
                }
            }

            SettingsTitleRow { title: loc!("pomodoro-timer") }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: loc!("countdown-timer"),
                    toggled: pomodoro,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_pomodoro(&!pomodoro) {
                            *state::SETTINGS.write() = settings_clone;
                            *state::TIMER_TEXT.write() = timer::get_timer_text(0);
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("timer-length"),
                    value: pomodoro_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone.change_pomodoro_length(&(pomodoro_length + delta)) {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("break-length"),
                    value: pomodoro_break_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone
                            .change_pomodoro_break_length(&(pomodoro_break_length + delta))
                        {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("snooze-length"),
                    value: pomodoro_snooze_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone
                            .change_pomodoro_snooze_length(&(pomodoro_snooze_length + delta))
                        {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("extended-breaks"),
                    toggled: pomodoro_extended_breaks,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone
                            .change_pomodoro_extended_breaks(&!pomodoro_extended_breaks)
                        {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("extended-break-interval"),
                    value: pomodoro_extended_break_interval,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone
                            .change_pomodoro_extended_break_interval(
                                &(pomodoro_extended_break_interval + delta),
                            )
                        {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("extended-break-length"),
                    value: pomodoro_extended_break_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone
                            .change_pomodoro_extended_break_length(
                                &(pomodoro_extended_break_length + delta),
                            )
                        {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
            }

            SettingsTitleRow { title: loc!("interface") }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: loc!("show-delete-confirmation"),
                    toggled: show_delete_confirmation,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone
                            .change_show_delete_confirmation(&!show_delete_confirmation)
                        {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
            }

            SettingsTitleRow { title: loc!("task-history") }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: loc!("show-project"),
                    toggled: show_task_project,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_task_project(&!show_task_project) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-tags"),
                    toggled: show_task_tags,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_task_tags(&!show_task_tags) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-earnings"),
                    toggled: show_task_earnings,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_task_earnings(&!show_task_earnings) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-seconds"),
                    toggled: show_seconds,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_seconds(&!show_seconds) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-daily-time-total"),
                    toggled: show_daily_time_total,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone
                            .change_show_daily_time_total(&!show_daily_time_total)
                        {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("dynamic-time-total"), // TODO: Add sublabel
                    toggled: dynamic_total,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_daily_time_total(&!dynamic_total) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsNumberRow {
                    label: loc!("days-to-show"),
                    value: days_to_show,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        match settings_clone.change_pomodoro_length(&(days_to_show + delta)) {
                            Ok(_) => *state::SETTINGS.write() = settings_clone,
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
            }

            SettingsTitleRow { title: loc!("todos") }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: loc!("show-project"),
                    toggled: show_todo_project,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_todo_project(&!show_todo_project) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-tags"),
                    toggled: show_todo_tags,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_todo_tags(&!show_todo_tags) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-earnings"),
                    toggled: show_todo_rate,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_todo_rate(&!show_todo_rate) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
            }

            SettingsTitleRow { title: loc!("idle") }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: loc!("show-tags"),
                    toggled: show_todo_tags,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_todo_tags(&!show_todo_tags) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
                SettingsToggleRow {
                    label: loc!("show-earnings"),
                    toggled: show_todo_rate,
                    onchange: move |_| {
                        let mut settings_clone = state::SETTINGS.cloned();
                        if let Ok(_) = settings_clone.change_show_todo_rate(&!show_todo_rate) {
                            *state::SETTINGS.write() = settings_clone;
                        }
                    },
                }
            }

            // Activate in a future release
            /*
            SettingsTitleRow { title: loc!("csv") }
            div { class: "settings-group",
                SettingsButtonRow {
                    label: loc!("import-csv"),
                    dangerous: false,
                    onclick: move |_| {},
                }
                SettingsButtonRow {
                    label: loc!("export-csv"),
                    dangerous: false,
                    onclick: move |_| {},
                }
            }

            SettingsTitleRow { title: loc!("danger-zone") }
            div { class: "settings-group",
                SettingsButtonRow {
                    label: loc!("delete-everything"),
                    dangerous: true,
                    onclick: move |_| {},
                }
            }*/
        }
    }
}

#[component]
fn SettingsTitleRow(title: String) -> Element {
    rsx! {
        div { class: "settings-header", "{title}" }
    }
}

#[component]
fn SettingsToggleRow(
    label: String,
    sublabel: Option<String>,
    toggled: bool,
    onchange: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div { class: "settings-item",
            div { class: "settings-label", "{label}" }
            label { class: "switch",
                input {
                    r#type: "checkbox",
                    checked: toggled,
                    onchange: move |e| onchange.call(e),
                }
                span { class: "slider round" }
            }
        }
    }
}

#[component]
fn SettingsNumberRow(
    label: String,
    sublabel: Option<String>,
    value: i64,
    onupdate: EventHandler<(i64, MouseEvent)>,
) -> Element {
    rsx! {
        div { class: "settings-item",
            div { class: "settings-label", "{label}" }
            div { class: "number-input",
                div { class: "value-display", id: "interval-value", "{value}" }
                div { class: "number-controls",
                    button {
                        class: "number-btn decrease",
                        onclick: move |event| onupdate.call((-1, event)),
                        "−"
                    }
                    button {
                        class: "number-btn increase",
                        onclick: move |event| onupdate.call((1, event)),
                        "+"
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsDropDownRow(
    label: String,
    // sublabel: Option<String>,
    list_items: Vec<String>,
    selected_item: String,
    onchange: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div { class: "settings-item",
            div { class: "settings-label", "{label}" }
            select {
                class: "settings-drop-down",
                onchange: move |e| onchange.call(e),
                for list_item in list_items {
                    option {
                        value: list_item.clone(),
                        selected: selected_item == list_item,
                        "{list_item}"
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsInputRow(
    label: String,
    input_type: String,
    // sublabel: Option<String>,
    value: String,
    placeholder: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div { class: "settings-item",
            div { class: "settings-label", "{label}" }
            input {
                class: "settings-input",
                r#type: input_type,
                value,
                oninput,
                placeholder,
            }
        }
    }
}

#[component]
fn SettingsButtonRow(label: String, dangerous: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        div { class: "settings-item",
            button {
                class: if dangerous { "settings-button dangerous" } else { "settings-button" },
                onclick,
                "{label}"
            }
        }
    }
}
