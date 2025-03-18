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

use crate::{constants::SETTINGS_CSS, state};

#[component]
pub fn SettingsView() -> Element {
    let mut state = use_context::<state::FurState>();
    let current_value = state.settings.read().pomodoro;
    rsx! {
        document::Stylesheet { href: SETTINGS_CSS }

        div { id: "settings",
            // SettingsTitleRow { title: "Pomodoro Timer".to_string() }
            // div { class: "settings-group",
            //     SettingsToggleRow {
            //         label: "Countdown timer".to_string(),
            //         toggled: from_settings(|settings| settings.pomodoro.clone()),
            //         onchange: move |_| from_settings_mut(|settings| settings.pomodoro = !settings.pomodoro),
            //     }
            //     SettingsToggleRow {
            //         label: "Extended break".to_string(),
            //         toggled: from_settings(|settings| settings.pomodoro.clone()),
            //         onchange: move |_| from_settings_mut(|settings| settings.pomodoro = !settings.pomodoro),
            //     }
            //     SettingsToggleRow {
            //         label: "Extended break".to_string(),
            //         toggled: from_settings(|settings| settings.pomodoro.clone()),
            //         onchange: move |_| from_settings_mut(|settings| settings.pomodoro = !settings.pomodoro),
            //     }
            // }

            SettingsTitleRow { title: "Extended Break".to_string() }
            div { class: "settings-group",
                SettingsToggleRow {
                    label: "Extended break".to_string(),
                    toggled: current_value,
                    onchange: move |_| {
                        let mut settings_clone = state.settings.read().clone();
                        if let Ok(_) = settings_clone.change_pomodoro(&!current_value) {
                            state.settings.set(settings_clone);
                        }
                    },
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
fn SettingsToggleRow(label: String, toggled: bool, onchange: EventHandler<FormEvent>) -> Element {
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
