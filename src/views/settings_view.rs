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
        server::{login::login_button_pressed, logout::logout_button_pressed, sync::request_sync},
        views::{settings::ServerChoices, timer},
    },
    state,
};

#[component]
pub fn SettingsView() -> Element {
    let mut state = use_context::<state::FurState>();

    // let user = state.user.read();
    // let user_fields = state.user_fields.read();

    let pomodoro = state.settings.read().pomodoro;
    let pomodoro_length = state.settings.read().pomodoro_length;
    let pomodoro_break_length = state.settings.read().pomodoro_break_length;
    let pomodoro_snooze_length = state.settings.read().pomodoro_snooze_length;
    let pomodoro_extended_breaks = state.settings.read().pomodoro_extended_breaks;
    let pomodoro_extended_break_interval = state.settings.read().pomodoro_extended_break_interval;
    let pomodoro_extended_break_length = state.settings.read().pomodoro_extended_break_length;

    let show_delete_confirmation = state.settings.read().show_delete_confirmation;

    let show_task_project = state.settings.read().show_task_project;
    let show_task_tags = state.settings.read().show_task_tags;
    let show_task_earnings = state.settings.read().show_task_earnings;
    let show_seconds = state.settings.read().show_seconds;
    let show_daily_time_total = state.settings.read().show_daily_time_total;
    let dynamic_total = state.settings.read().dynamic_total;
    let days_to_show = state.settings.read().days_to_show;

    let show_todo_project = state.settings.read().show_todo_project;
    let show_todo_tags = state.settings.read().show_todo_tags;
    let show_todo_rate = state.settings.read().show_todo_rate;

    // let mut sync_button_row: Row<'_, Message> =
    //     row![button(text(self.localization.get_message(
    //         if self.fur_user.is_none() {
    //             "log-in"
    //         } else {
    //             "log-out"
    //         },
    //         None
    //     )))
    //     .on_press_maybe(if self.fur_user.is_none() {
    //         if !self.fur_user_fields.server.is_empty()
    //             && !self.fur_user_fields.email.is_empty()
    //             && !self.fur_user_fields.encryption_key.is_empty()
    //         {
    //             Some(Message::UserLoginPressed)
    //         } else {
    //             None
    //         }
    //     } else {
    //         Some(Message::UserLogoutPressed)
    //     })
    //     .style(if self.fur_user.is_none() {
    //         style::primary_button_style
    //     } else {
    //         button::secondary
    //     }),]
    //     .spacing(10);
    // sync_button_row = sync_button_row.push_maybe(if self.fur_user.is_some() {
    //     Some(
    //         button(text(self.localization.get_message("sync", None)))
    // .on_press_maybe(match self.fur_user {
    //     Some(_) => {
    //         if self.login_message.iter().any(|message| {
    //             message != &self.localization.get_message("syncing", None)
    //         }) {
    //             Some(Message::SyncWithServer)
    //         } else {
    //             None
    //         }
    //     }
    //     None => None,
    // })
    //             .style(style::primary_button_style),
    //     )
    // } else {
    //     Some(
    //         button(text(self.localization.get_message("sign-up", None)))
    //             .on_press(Message::OpenUrl("https://furtherance.app/sync".to_string()))
    //             .style(style::primary_button_style),
    //     )
    // });
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

        // TODO: Localize labels/text

        div { id: "settings",
            SettingsTitleRow { title: "Sync".to_string() }
            div { class: "settings-group",
                SettingsDropDownRow {
                    label: "Server".to_string(),
                    list_items: ServerChoices::all_as_strings(),
                    selected_item: state.user_fields.read().server_selection.to_string(),
                    onchange: move |event: Event<FormData>| {
                        let mut user_fields_clone = state.user_fields.read().clone();
                        if event.data.value() == ServerChoices::Official.to_string() {
                            user_fields_clone.server_selection = ServerChoices::Official;
                            user_fields_clone.server = OFFICIAL_SERVER.to_string();
                        } else {
                            user_fields_clone.server_selection = ServerChoices::Custom;
                            if let Some(fur_user) = state.user.read().clone() {
                                user_fields_clone.server = fur_user.server;
                            } else {
                                user_fields_clone.server = String::new();
                            }
                        }
                        state.user_fields.set(user_fields_clone);
                    },
                }
                SettingsInputRow {
                    label: "Email".to_string(),
                    input_type: "email".to_string(),
                    value: state.user_fields.read().email.clone(),
                    placeholder: "Email address".to_string(),
                    oninput: move |event: Event<FormData>| {
                        let mut user_fields_clone = state.user_fields.read().clone();
                        user_fields_clone.email = event.value();
                        state.user_fields.set(user_fields_clone);
                    },
                }
                SettingsInputRow {
                    label: "Encryption key".to_string(),
                    input_type: "password".to_string(),
                    value: state.user_fields.read().encryption_key.clone(),
                    placeholder: "Key".to_string(),
                    oninput: move |event: Event<FormData>| {
                        let mut user_fields_clone = state.user_fields.read().clone();
                        user_fields_clone.encryption_key = event.value();
                        state.user_fields.set(user_fields_clone);
                    },
                }
                SettingsButtonRow {
                    label: if state.user.read().is_some() { "Log out".to_string() } else { "Log in".to_string() },
                    dangerous: false,
                    onclick: move |_| {
                        if state.user.read().is_some() {
                            logout_button_pressed();
                        } else {
                            let user_fields = state.user_fields.read().clone();
                            if !user_fields.server.is_empty() && !user_fields.email.is_empty()
                                && !user_fields.encryption_key.is_empty()
                            {
                                login_button_pressed();
                            }
                        }
                    },
                }
                SettingsButtonRow {
                    label: "Sync".to_string(),
                    dangerous: false,
                    onclick: move |_| {
                        if state.user.read().is_some() {
                            request_sync();
                        }
                    },
                }
                SettingsButtonRow {
                    label: "Log out".to_string(),
                    dangerous: true,
                    onclick: move |_| {},
                }
            }

            SettingsTitleRow { title: "Pomodoro Timer".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Countdown timer".to_string(),
                    toggled: pomodoro,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_pomodoro(&!pomodoro) {
                            state.settings.set(settings_clone);
                            *state::TIMER_TEXT.write() = timer::get_timer_text(0);
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Timer length".to_string(),
                    value: pomodoro_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone.change_pomodoro_length(&(pomodoro_length + delta)) {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Break length".to_string(),
                    value: pomodoro_break_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone
                            .change_pomodoro_break_length(&(pomodoro_break_length + delta))
                        {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Snooze length".to_string(),
                    value: pomodoro_snooze_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone
                            .change_pomodoro_snooze_length(&(pomodoro_snooze_length + delta))
                        {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Extended breaks".to_string(),
                    toggled: pomodoro_extended_breaks,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone
                            .change_pomodoro_extended_breaks(&!pomodoro_extended_breaks)
                        {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Extended break interval".to_string(),
                    value: pomodoro_extended_break_interval,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone
                            .change_pomodoro_extended_break_interval(
                                &(pomodoro_extended_break_interval + delta),
                            )
                        {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Extended break length".to_string(),
                    value: pomodoro_extended_break_length,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone
                            .change_pomodoro_extended_break_length(
                                &(pomodoro_extended_break_length + delta),
                            )
                        {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
            }

            SettingsTitleRow { title: "Interface".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Show delete confirmation".to_string(),
                    toggled: show_delete_confirmation,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone
                            .change_show_delete_confirmation(&!show_delete_confirmation)
                        {
                            state.settings.set(settings_clone);
                        }
                    },
                }
            }

            SettingsTitleRow { title: "Task History".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Show project".to_string(),
                    toggled: show_task_project,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_task_project(&!show_task_project) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show tags".to_string(),
                    toggled: show_task_tags,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_task_tags(&!show_task_tags) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show earnings".to_string(),
                    toggled: show_task_earnings,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_task_earnings(&!show_task_earnings) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show seconds".to_string(),
                    toggled: show_seconds,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_seconds(&!show_seconds) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show daily time total".to_string(),
                    toggled: show_daily_time_total,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone
                            .change_show_daily_time_total(&!show_daily_time_total)
                        {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Dynamic total".to_string(), // TODO: Add sublabel
                    toggled: dynamic_total,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_daily_time_total(&!dynamic_total) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsNumberRow {
                    label: "Days to show".to_string(),
                    value: days_to_show,
                    onupdate: move |(delta, _)| {
                        let mut settings_clone = state.settings.read().clone();
                        match settings_clone.change_pomodoro_length(&(days_to_show + delta)) {
                            Ok(_) => state.settings.set(settings_clone),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    },
                }
            }

            SettingsTitleRow { title: "Todos".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Show project".to_string(),
                    toggled: show_todo_project,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_todo_project(&!show_todo_project) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show tags".to_string(),
                    toggled: show_todo_tags,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_todo_tags(&!show_todo_tags) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show earnings".to_string(),
                    toggled: show_todo_rate,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_todo_rate(&!show_todo_rate) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
            }

            SettingsTitleRow { title: "Idle".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Show tags".to_string(),
                    toggled: show_todo_tags,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_todo_tags(&!show_todo_tags) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
                SettingsToggleRow {
                    label: "Show earnings".to_string(),
                    toggled: show_todo_rate,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_show_todo_rate(&!show_todo_rate) {
                            state.settings.set(settings_clone);
                        }
                    },
                }
            }

            // TODO: Make these function (need access to file browser/share sheet)
            SettingsTitleRow { title: "CSV".to_string() }
            div { class: "settings-group",
                SettingsButtonRow {
                    label: "Import CSV".to_string(),
                    dangerous: false,
                    onclick: move |_| {},
                }
                SettingsButtonRow {
                    label: "Export CSV".to_string(),
                    dangerous: false,
                    onclick: move |_| {},
                }
            }

            SettingsTitleRow { title: "Danger Zone".to_string() }
            div { class: "settings-group",
                SettingsButtonRow {
                    label: "Delete Everything".to_string(),
                    dangerous: true,
                    onclick: move |_| {},
                }
            }
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
