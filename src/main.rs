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

mod models {
    pub mod fur_alert;
    pub mod fur_persist;
    pub mod fur_pomodoro;
    pub mod fur_settings;
    pub mod fur_sheet;
    pub mod fur_shortcut;
    pub mod fur_task;
    pub mod fur_task_group;
    pub mod fur_todo;
    pub mod fur_user;
}
pub mod database {
    pub mod init;
    pub mod persistence;
    pub mod shortcuts;
    pub mod sync;
    pub mod tasks;
    pub mod todos;
}
mod helpers {
    pub mod actions;
    pub mod color_utils;
    pub mod formatters;
    pub mod icons;
    pub mod view_enums;
    pub mod views {
        pub mod settings;
        pub mod shortcuts;
        pub mod task_history;
        pub mod task_input;
        pub mod timer;
        pub mod todos;
    }
    pub mod server {
        pub mod encryption;
        pub mod login;
        pub mod logout;
        pub mod sync;
    }
}
mod views {
    pub mod settings_view;
    pub mod shortcuts_view;
    pub mod timer_view;
    pub mod todos_view;
}
mod constants;
mod localization;
mod state;

use constants::{ALERT_CSS, MAIN_CSS, TIMER_CSS};
use database::init::db_init;
use dioxus::prelude::*;
use helpers::{
    formatters,
    icons::{BsBookmark, BsCheck2Circle, BsGear, BsHourglassSplit, Icon, IconShape},
    server::sync::schedule_sync,
    views::timer::ensure_timer_running,
};
use state::ACTIVE_TAB;
use views::{
    settings_view::SettingsView, shortcuts_view::ShortcutsView, timer_view::TimerView,
    todos_view::TodosView,
};

#[derive(PartialEq, Copy, Clone)]
pub enum NavTab {
    Timer,
    Todos,
    Shortcuts,
    Settings,
}

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    match database::persistence::retrieve_persisting_timer() {
        Ok(persisting_timer) => {
            if persisting_timer.is_running {
                *state::TIMER_IS_RUNNING.write() = persisting_timer.is_running;
                *state::TASK_INPUT.write() = persisting_timer.task_input;
                *state::TIMER_START_TIME.write() = persisting_timer.start_time;
            }
        }
        Err(e) => eprintln!("Error retrieving persisting timer: {}", e),
    }
    ensure_timer_running();
    schedule_sync();

    let alert = state::ALERT.cloned();
    let sheets = state::SHEETS.cloned();

    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: TIMER_CSS }
        document::Stylesheet { href: ALERT_CSS }

        div {
            id: "page-content",
            match ACTIVE_TAB.cloned() {
                NavTab::Timer => rsx! {
                    TimerView {}
                },
                NavTab::Todos => rsx! {
                    TodosView {}
                },
                NavTab::Shortcuts => rsx! {
                    ShortcutsView {}
                },
                NavTab::Settings => rsx! {
                    SettingsView {}
                },
            }
        }

        if !sheets.new_task_is_shown && !sheets.new_shortcut_is_shown
            && !sheets.new_todo_is_shown && sheets.task_edit_sheet.is_none()
            && sheets.group_details_sheet.is_none() && sheets.edit_todo_sheet.is_none()
            && sheets.edit_shortcut_sheet.is_none()
        {
            BottomNav {}
        }

        if alert.is_shown {
            AlertDialog {
                title: alert.title,
                message: alert.message,
                confirm_button: alert.confirm_button,
                cancel_button: alert.cancel_button,
            }
        }
    }
}

#[component]
pub fn BottomNav() -> Element {
    let active_tab = ACTIVE_TAB.cloned();
    rsx! {
        div { class: "bottom-nav",
            NavItem {
                icon: BsHourglassSplit,
                label: "Timer",
                active: active_tab == NavTab::Timer,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Timer,
            }
            NavItem {
                icon: BsCheck2Circle,
                label: "Todos",
                active: active_tab == NavTab::Todos,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Todos,
            }
            NavItem {
                icon: BsBookmark,
                label: "Shortcuts",
                active: active_tab == NavTab::Shortcuts,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Shortcuts,
            }
            NavItem {
                icon: BsGear,
                label: "Settings",
                active: active_tab == NavTab::Settings,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Settings,
            }
        }
    }
}

#[component]
fn NavItem<I: IconShape + Clone + PartialEq + 'static>(
    icon: I,
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if active {
        "nav-item active"
    } else {
        "nav-item"
    };

    rsx! {
        div { class: "{class}", onclick: move |e| onclick.call(e),
            Icon { icon, width: 25, height: 25 }
            span { class: "nav-label", "{label}" }
        }
    }
}

#[component]
fn AlertDialog(
    title: String,
    message: String,
    confirm_button: (String, fn()),
    cancel_button: Option<(String, fn())>,
) -> Element {
    rsx! {
        div { class: "modal-overlay",
            div { class: "dialog-container",
                div { class: "dialog-content",
                    div { class: "dialog-title", "{title}" }
                    div { class: "dialog-message", "{message}" }
                }
                div { class: "dialog-buttons",
                    if let Some((button_text, button_click_event)) = cancel_button {
                        button {
                            class: "dialog-button cancel-button",
                            onclick: move |_| (button_click_event)(),
                            "{button_text}"
                        }
                    }
                    button {
                        class: "dialog-button confirm-button",
                        onclick: move |_| (confirm_button.1)(),
                        "{confirm_button.0}"
                    }
                }
            }
        }
    }
}
