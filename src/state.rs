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
    helpers::{tasks, views::todos},
    models::{fur_task_group::FurTaskGroup, fur_todo::FurTodo},
    NavTab,
};

pub static ACTIVE_TAB: GlobalSignal<NavTab> = Global::new(|| NavTab::Timer);
pub static TIMER_TEXT: GlobalSignal<String> = Global::new(|| "0:00:00".to_string());
pub static TIMER_IS_RUNNING: GlobalSignal<bool> = Global::new(|| false);
pub static TASK_INPUT: GlobalSignal<String> = Global::new(|| String::new());
pub static TIMER_START_TIME: GlobalSignal<DateTime<Local>> = Global::new(|| Local::now());

#[derive(Debug, Clone, Copy)]
pub struct TaskHistory {
    pub sorted: Signal<BTreeMap<NaiveDate, Vec<FurTaskGroup>>>,
}

pub fn use_task_history_provider() {
    let sorted = use_signal(|| tasks::get_task_history(365));
    use_context_provider(|| TaskHistory { sorted });
}

#[derive(Debug, Clone, Copy)]
pub struct AllTodos {
    pub sorted: Signal<BTreeMap<NaiveDate, Vec<FurTodo>>>,
}

pub fn use_all_todos_provider() {
    let sorted = use_signal(|| todos::get_all_todos());
    use_context_provider(|| AllTodos { sorted });
}
