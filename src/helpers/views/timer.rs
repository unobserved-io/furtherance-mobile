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

use std::time::Duration;

use chrono::{DateTime, Local};
use dioxus::{
    hooks::{use_coroutine, Coroutine, UnboundedReceiver},
    prelude::spawn,
    signals::Readable,
};

use crate::{formatters, helpers::database, models::fur_task::FurTask, state};

pub fn stop_timer(stop_time: DateTime<Local>) {
    *state::TIMER_IS_RUNNING.write() = false;

    let (name, project, tags, rate) = formatters::split_task_input(&state::TASK_INPUT.cloned());
    database::insert_task(&FurTask::new(
        name,
        state::TIMER_START_TIME.cloned(),
        stop_time,
        tags,
        project,
        rate,
        String::new(),
    ))
    .expect("Couldn't write task to database.");

    // TODO: - delete_autosave();
    reset_timer();
}

pub fn start_timer() {
    *state::TIMER_START_TIME.write() = Local::now();
    *state::TIMER_IS_RUNNING.write() = true;
    // TODO: Pomodoro
    // if state.fur_settings.pomodoro && !state.pomodoro.on_break {
    //     state.pomodoro.sessions += 1;
    // }

    spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        loop {
            if state::TIMER_IS_RUNNING.cloned() {
                let duration = Local::now().signed_duration_since(state::TIMER_START_TIME.cloned());
                let seconds_elapsed = duration.num_seconds();
                *state::TIMER_TEXT.write() = get_timer_text(seconds_elapsed);

                // TODO: Pomodoro
                // if self.fur_settings.pomodoro
                //     && self.timer_text == "0:00:00".to_string()
                //     && seconds_elapsed > 2
                // {
                //     // Check if idle or other alert is being displayed so as not to replace it
                //     if self.displayed_alert.is_none() {
                //         if self.pomodoro.on_break {
                //             show_notification(
                //                 NotificationType::BreakOver,
                //                 &self.localization,
                //                 self.fur_settings.pomodoro_notification_alarm_sound,
                //             );
                //             self.displayed_alert = Some(FurAlert::PomodoroBreakOver);
                //         } else {
                //             show_notification(
                //                 NotificationType::PomodoroOver,
                //                 &self.localization,
                //                 self.fur_settings.pomodoro_notification_alarm_sound,
                //             );
                //             self.displayed_alert = Some(FurAlert::PomodoroOver);
                //         }
                //     }
                //     return Task::none();
                // }

                // TODO:
                // if self.fur_settings.notify_on_idle
                //     && self.displayed_alert != Some(FurAlert::PomodoroOver)
                // {
                //     let idle_time = idle::get_idle_time() as i64;
                //     if idle_time >= self.fur_settings.chosen_idle_time * 60
                //         && !self.idle.reached
                //     {
                //         // User is idle
                //         self.idle.reached = true;
                //         self.idle.start_time = Local::now()
                //             - TimeDelta::seconds(self.fur_settings.chosen_idle_time * 60);
                //     } else if idle_time < self.fur_settings.chosen_idle_time * 60
                //         && self.idle.reached
                //         && !self.idle.notified
                //     {
                //         // User is back - show idle message
                //         self.idle.notified = true;
                //         show_notification(
                //             NotificationType::Idle,
                //             &self.localization,
                //             self.fur_settings.pomodoro_notification_alarm_sound,
                //         );
                //         self.displayed_alert = Some(FurAlert::Idle);
                //     }
                // }

                // TODO: Write autosave every minute
                // if seconds_elapsed > 1 && seconds_elapsed % 60 == 0 {
                //     if let Err(e) = write_autosave(&self.task_input, self.timer_start_time) {
                //         eprintln!("Error writing autosave: {e}");
                //     }
                // }
            } else {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}

fn reset_timer() {
    *state::TASK_INPUT.write() = String::new();
    *state::TIMER_TEXT.write() = get_timer_text(0);
    // state.idle = FurIdle::new();
}

fn get_timer_text(seconds_elapsed: i64) -> String {
    if state::TIMER_IS_RUNNING.cloned() {
        get_running_timer_text(seconds_elapsed)
    } else {
        get_stopped_timer_text()
    }
}

fn get_running_timer_text(seconds_elapsed: i64) -> String {
    // TODO: Pomodoro
    // if state.fur_settings.pomodoro {
    //     let stop_time = if state.pomodoro.on_break {
    //         if state.fur_settings.pomodoro_extended_breaks
    //             && state.pomodoro.sessions % state.fur_settings.pomodoro_extended_break_interval
    //                 == 0
    //         {
    //             state.timer_start_time
    //                 + TimeDelta::minutes(state.fur_settings.pomodoro_extended_break_length)
    //         } else {
    //             state.timer_start_time
    //                 + TimeDelta::minutes(state.fur_settings.pomodoro_break_length)
    //         }
    //     } else {
    //         if state.pomodoro.snoozed {
    //             state.pomodoro.snoozed_at
    //                 + TimeDelta::minutes(state.fur_settings.pomodoro_snooze_length)
    //         } else {
    //             state.timer_start_time + TimeDelta::minutes(state.fur_settings.pomodoro_length)
    //         }
    //     };

    //     let seconds_until_end =
    //         (stop_time - state.timer_start_time).num_seconds() - seconds_elapsed;
    //     if seconds_until_end > 0 {
    //         seconds_to_formatted_duration(seconds_until_end, true)
    //     } else {
    //         "0:00:00".to_string()
    //     }
    // } else {
    seconds_to_formatted_duration(seconds_elapsed, true)
    // }
}

fn get_stopped_timer_text() -> String {
    // TODO: Pomodoro
    // if state.fur_settings.pomodoro {
    //     if state.pomodoro.on_break {
    //         if state.fur_settings.pomodoro_extended_breaks
    //             && state.pomodoro.sessions % state.fur_settings.pomodoro_extended_break_interval
    //                 == 0
    //         {
    //             seconds_to_formatted_duration(
    //                 state.fur_settings.pomodoro_extended_break_length * 60,
    //                 true,
    //             )
    //         } else {
    //             seconds_to_formatted_duration(state.fur_settings.pomodoro_break_length * 60, true)
    //         }
    //     } else if state.pomodoro.snoozed {
    //         seconds_to_formatted_duration(state.fur_settings.pomodoro_snooze_length * 60, true)
    //     } else {
    //         seconds_to_formatted_duration(state.fur_settings.pomodoro_length * 60, true)
    //     }
    // } else {
    "0:00:00".to_string()
    // }
}

fn seconds_to_formatted_duration(total_seconds: i64, show_seconds: bool) -> String {
    if show_seconds {
        seconds_to_hms(total_seconds)
    } else {
        seconds_to_hm(total_seconds)
    }
}

fn seconds_to_hms(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    let s = total_seconds % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}

fn seconds_to_hm(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    format!("{}:{:02}", h, m)
}
