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

use std::collections::BTreeMap;

use chrono::{DateTime, Local, NaiveDate};
use dioxus::{
    hooks::{use_context_provider, use_signal},
    signals::{Global, GlobalSignal, Signal},
};

use crate::{
    helpers::{
        server,
        views::{shortcuts, task_history, todos},
    },
    models::{
        fur_alert::FurAlert,
        fur_pomodoro::FurPomodoro,
        fur_settings::FurSettings,
        fur_sheet::FurSheet,
        fur_shortcut::FurShortcut,
        fur_task_group::FurTaskGroup,
        fur_todo::FurTodo,
        fur_user::{FurUser, FurUserFields},
    },
    NavTab,
};

pub static ACTIVE_TAB: GlobalSignal<NavTab> = Global::new(|| NavTab::Timer);
pub static TIMER_TEXT: GlobalSignal<String> = Global::new(|| "0:00:00".to_string());
pub static TIMER_IS_RUNNING: GlobalSignal<bool> = Global::new(|| false);
pub static TASK_INPUT: GlobalSignal<String> = Global::new(|| String::new());
pub static TIMER_START_TIME: GlobalSignal<DateTime<Local>> = Global::new(|| Local::now());

#[derive(Debug, Clone, Copy)]
pub struct FurState {
    pub alert: Signal<FurAlert>,
    pub pomodoro: Signal<FurPomodoro>,
    pub settings: Signal<FurSettings>,
    pub sheets: Signal<FurSheet>,
    pub shortcuts: Signal<Vec<FurShortcut>>,
    pub tasks: Signal<BTreeMap<NaiveDate, Vec<FurTaskGroup>>>,
    pub todos: Signal<BTreeMap<NaiveDate, Vec<FurTodo>>>,
    pub user: Signal<Option<FurUser>>,
    pub user_fields: Signal<FurUserFields>,
}

pub fn use_state_provider() {
    let alert = use_signal(|| FurAlert::new());
    let pomodoro = use_signal(|| FurPomodoro::new());
    let settings = use_signal(|| FurSettings::new().expect("Failed to load settings"));
    let sheets = use_signal(|| FurSheet::new());
    let shortcuts = use_signal(|| shortcuts::get_all_shortcuts());
    let tasks = use_signal(|| task_history::get_task_history(365));
    let todos = use_signal(|| todos::get_all_todos());
    let user = use_signal(|| server::sync::get_user());
    let user_fields = use_signal(|| server::sync::get_user_fields());

    use_context_provider(|| FurState {
        alert,
        pomodoro,
        settings,
        sheets,
        shortcuts,
        tasks,
        todos,
        user,
        user_fields,
    });
}
