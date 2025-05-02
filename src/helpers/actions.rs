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

use chrono::Local;
use dioxus::signals::Readable;

use crate::{
    helpers::views::timer::{reset_timer, stop_timer},
    state, NavTab,
};

use super::{
    server::sync::sync_after_change,
    views::{task_history, timer},
};

pub fn start_stop_pressed() {
    if state::TIMER_IS_RUNNING.cloned() {
        let mut pomodoro = state::POMODORO.cloned();
        // Do not move declarations to after if else
        // They are needed in this position to properly initiate timer on reset
        if pomodoro.on_break {
            *state::TIMER_IS_RUNNING.write() = false;
            pomodoro.on_break = false;
            pomodoro.snoozed = false;
            pomodoro.sessions = 0;
            *state::POMODORO.write() = pomodoro.clone();
            reset_timer();
            task_history::update_task_history(state::SETTINGS.read().days_to_show);
        } else {
            pomodoro.on_break = false;
            pomodoro.snoozed = false;
            pomodoro.sessions = 0;
            *state::POMODORO.write() = pomodoro.clone();
            stop_timer(Local::now());
            task_history::update_task_history(state::SETTINGS.read().days_to_show);

            sync_after_change();
        }
    } else {
        timer::start_timer();
    }
}

pub fn start_timer_with_task(task_text: String) {
    if !state::TIMER_IS_RUNNING.cloned() {
        *state::TASK_INPUT.write() = task_text;
        timer::start_timer();
        *state::ACTIVE_TAB.write() = NavTab::Timer;
    }
}
