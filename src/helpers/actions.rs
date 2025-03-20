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
use dioxus::{hooks::use_context, signals::Readable};

use crate::{state, NavTab};

use super::views::{task_history, timer};

pub fn start_stop_pressed() {
    if state::TIMER_IS_RUNNING.cloned() {
        // Do not move declarations to after if else
        // They are needed in this position to properly initiate timer on reset

        // TODO: Pomodoro
        // if self.pomodoro.on_break {
        //     self.timer_is_running = false;
        //     self.pomodoro.on_break = false;
        //     self.pomodoro.snoozed = false;
        //     self.pomodoro.sessions = 0;
        //     reset_timer(self);
        //     return messages::update_task_history(self.fur_settings.days_to_show);
        // } else {
        //     self.pomodoro.on_break = false;
        //     self.pomodoro.snoozed = false;
        //     self.pomodoro.sessions = 0;
        timer::stop_timer(Local::now());

        task_history::update_task_history(
            use_context::<state::FurState>()
                .settings
                .read()
                .days_to_show,
        );
        // TODO: Sync after change - tasks.push(messages::sync_after_change(&self.fur_user));
        // }
    } else {
        timer::start_timer();
    }
}

pub fn start_timer_with_task(task_text: String) {
    if !state::TIMER_IS_RUNNING.cloned() {
        *state::TASK_INPUT.write() = task_text;
        // TODO:
        // self.inspector_view = None;
        // self.task_to_add = None;
        // self.task_to_edit = None;
        timer::start_timer();
        *state::ACTIVE_TAB.write() = NavTab::Timer;
    }
}
