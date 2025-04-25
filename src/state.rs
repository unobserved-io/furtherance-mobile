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

pub static SETTINGS: GlobalSignal<FurSettings> =
    Global::new(|| FurSettings::new().expect("Failed to load settings"));
pub static SHORTCUTS: GlobalSignal<Vec<FurShortcut>> =
    Global::new(|| shortcuts::get_all_shortcuts());
pub static TASKS: GlobalSignal<BTreeMap<NaiveDate, Vec<FurTaskGroup>>> =
    Global::new(|| task_history::get_task_history(365));
pub static TODOS: GlobalSignal<BTreeMap<NaiveDate, Vec<FurTodo>>> =
    Global::new(|| todos::get_all_todos());
pub static USER: GlobalSignal<Option<FurUser>> = Global::new(|| server::sync::get_user());
pub static SYNC_MESSAGE: GlobalSignal<Result<String, Box<dyn std::error::Error>>> =
    Global::new(|| Ok(String::new()));
pub static USER_FIELDS: GlobalSignal<FurUserFields> =
    Global::new(|| server::sync::get_user_fields());
pub static ALERT: GlobalSignal<FurAlert> = Global::new(|| FurAlert::new());
pub static POMODORO: GlobalSignal<FurPomodoro> = Global::new(|| FurPomodoro::new());
pub static SHEETS: GlobalSignal<FurSheet> = Global::new(|| FurSheet::new());

pub static ACTIVE_TAB: GlobalSignal<NavTab> = Global::new(|| NavTab::Timer);
pub static TIMER_TEXT: GlobalSignal<String> = Global::new(|| "0:00:00".to_string());
pub static TIMER_IS_RUNNING: GlobalSignal<bool> = Global::new(|| false);
pub static TASK_INPUT: GlobalSignal<String> = Global::new(|| String::new());
pub static TIMER_START_TIME: GlobalSignal<DateTime<Local>> = Global::new(|| Local::now());
pub static TASK_IDS_TO_DELETE: GlobalSignal<Option<Vec<String>>> = Global::new(|| None);
pub static SHORTCUT_ID_TO_DELETE: GlobalSignal<Option<String>> = Global::new(|| None);
pub static TODO_ID_TO_DELETE: GlobalSignal<Option<String>> = Global::new(|| None);
