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

use chrono::{DateTime, Local, TimeDelta};
use dioxus::{
    hooks::use_context,
    prelude::spawn,
    signals::{Readable, Writable},
};
use std::sync::Once;

use crate::{
    database, formatters,
    helpers::{server::sync::request_sync, views::task_history::update_task_history},
    loc,
    localization::Localization,
    models::fur_task::FurTask,
    state,
};

static TIMER_INIT: Once = Once::new();

pub fn ensure_timer_running() {
    TIMER_INIT.call_once(|| {
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                if state::TIMER_IS_RUNNING.cloned() {
                    let duration =
                        Local::now().signed_duration_since(state::TIMER_START_TIME.cloned());
                    let seconds_elapsed = duration.num_seconds();
                    *state::TIMER_TEXT.write() = get_timer_text(seconds_elapsed);

                    let mut state = use_context::<state::FurState>();
                    let settings = state.settings.read().clone();
                    let mut alert = state.alert.read().clone();
                    let pomodoro = state.pomodoro.read().clone();

                    if settings.pomodoro
                        && state::TIMER_TEXT.cloned() == "0:00:00".to_string()
                        && seconds_elapsed > 2
                    {
                        // Check if other alert is being displayed so as not to replace it
                        if !alert.is_shown {
                            if pomodoro.on_break {
                                // TODO: Show notification
                                // show_notification(
                                //     NotificationType::BreakOver,
                                //     &self.localization,
                                //     self.fur_settings.pomodoro_notification_alarm_sound,
                                // );
                                alert.is_shown = true;
                                alert.title = loc!("break-over-title");
                                alert.message = loc!("break-over-description");
                                alert.confirm_button =
                                    (loc!("continue"), || continue_after_break());
                                alert.cancel_button = Some((loc!("stop"), || stop_after_break()));
                                state.alert.set(alert);
                            } else {
                                // TODO: Show notification
                                // show_notification(
                                //     NotificationType::PomodoroOver,
                                //     &self.localization,
                                //     self.fur_settings.pomodoro_notification_alarm_sound,
                                // );
                                alert.is_shown = true;
                                alert.title = loc!("pomodoro-over-title");
                                alert.message = loc!("pomodoro-over-description");
                                alert.confirm_button = if settings.pomodoro_extended_breaks
                                    && pomodoro.sessions % settings.pomodoro_extended_break_interval
                                        == 0
                                {
                                    (loc!("long-break"), || start_break())
                                } else {
                                    (loc!("break"), || start_break())
                                };
                                alert.cancel_button =
                                    Some((loc!("stop"), || stop_pomodoro_timer()));
                                state.alert.set(alert);
                            }
                        }
                        continue;
                    }

                    // TODO: Write autosave every minute
                    // if seconds_elapsed > 1 && seconds_elapsed % 60 == 0 {
                    //     if let Err(e) = write_autosave(&self.task_input, self.timer_start_time) {
                    //         eprintln!("Error writing autosave: {e}");
                    //     }
                    // }
                }
            }
        });
    });
}

pub fn stop_timer(stop_time: DateTime<Local>) {
    *state::TIMER_IS_RUNNING.write() = false;

    let (name, project, tags, rate) = formatters::split_task_input(&state::TASK_INPUT.cloned());
    database::tasks::insert_task(&FurTask::new(
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
    let mut state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let mut pomodoro = state.pomodoro.read().clone();

    if settings.pomodoro && !pomodoro.on_break {
        pomodoro.sessions += 1;
        state.pomodoro.set(pomodoro);
    }

    ensure_timer_running();
}

fn reset_timer() {
    *state::TASK_INPUT.write() = String::new();
    *state::TIMER_TEXT.write() = get_timer_text(0);
}

pub fn get_timer_text(seconds_elapsed: i64) -> String {
    if state::TIMER_IS_RUNNING.cloned() {
        get_running_timer_text(seconds_elapsed)
    } else {
        get_stopped_timer_text()
    }
}

fn get_running_timer_text(seconds_elapsed: i64) -> String {
    let state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let pomodoro = state.pomodoro.read().clone();

    if settings.pomodoro {
        let stop_time = if pomodoro.on_break {
            if settings.pomodoro_extended_breaks
                && pomodoro.sessions % settings.pomodoro_extended_break_interval == 0
            {
                state::TIMER_START_TIME.cloned()
                    + TimeDelta::minutes(settings.pomodoro_extended_break_length)
            } else {
                state::TIMER_START_TIME.cloned()
                    + TimeDelta::minutes(settings.pomodoro_break_length)
            }
        } else {
            if pomodoro.snoozed {
                pomodoro.snoozed_at + TimeDelta::minutes(settings.pomodoro_snooze_length)
            } else {
                state::TIMER_START_TIME.cloned() + TimeDelta::minutes(settings.pomodoro_length)
            }
        };

        let seconds_until_end =
            (stop_time - state::TIMER_START_TIME.cloned()).num_seconds() - seconds_elapsed;
        if seconds_until_end > 0 {
            seconds_to_formatted_duration(seconds_until_end, true)
        } else {
            "0:00:00".to_string()
        }
    } else {
        seconds_to_formatted_duration(seconds_elapsed, true)
    }
}

pub fn get_stopped_timer_text() -> String {
    let state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let pomodoro = state.pomodoro.read().clone();

    if settings.pomodoro {
        if pomodoro.on_break {
            if settings.pomodoro_extended_breaks
                && pomodoro.sessions % settings.pomodoro_extended_break_interval == 0
            {
                seconds_to_formatted_duration(settings.pomodoro_extended_break_length * 60, true)
            } else {
                seconds_to_formatted_duration(settings.pomodoro_break_length * 60, true)
            }
        } else if pomodoro.snoozed {
            seconds_to_formatted_duration(settings.pomodoro_snooze_length * 60, true)
        } else {
            seconds_to_formatted_duration(settings.pomodoro_length * 60, true)
        }
    } else {
        "0:00:00".to_string()
    }
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

fn stop_pomodoro_timer() {
    let mut state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let mut alert = state.alert.read().clone();
    let mut pomodoro = state.pomodoro.read().clone();

    pomodoro.snoozed = false;
    stop_timer(Local::now());
    alert.is_shown = false;
    state.alert.set(alert);
    pomodoro.sessions = 0;
    state.pomodoro.set(pomodoro);
    update_task_history(settings.days_to_show);
    // TODO: Test if this updates state (i.e. if it loads a new task we didn't have):
    spawn(async move {
        request_sync();
    });
}

fn start_break() {
    let mut state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let mut alert = state.alert.read().clone();
    let mut pomodoro = state.pomodoro.read().clone();

    let original_task_input = state::TASK_INPUT.cloned();
    pomodoro.on_break = true;
    pomodoro.snoozed = false;
    state.pomodoro.set(pomodoro);
    stop_timer(Local::now());
    *state::TASK_INPUT.write() = original_task_input;
    alert.is_shown = false;
    state.alert.set(alert);
    start_timer();
    update_task_history(settings.days_to_show);
    // TODO: Test if this updates state (i.e. if it loads a new task we didn't have):
    spawn(async move {
        request_sync();
    });
}

fn stop_after_break() {
    let mut state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let mut alert = state.alert.read().clone();
    let mut pomodoro = state.pomodoro.read().clone();
    *state::TIMER_IS_RUNNING.write() = false;
    pomodoro.on_break = false;
    pomodoro.snoozed = false;
    pomodoro.sessions = 0;
    state.pomodoro.set(pomodoro);
    reset_timer();
    alert.is_shown = false;
    state.alert.set(alert);
    update_task_history(settings.days_to_show);
}

fn continue_after_break() {
    let mut state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    let mut alert = state.alert.read().clone();
    let mut pomodoro = state.pomodoro.read().clone();

    *state::TIMER_IS_RUNNING.write() = false;
    let original_task_input = state::TASK_INPUT.cloned();
    pomodoro.on_break = false;
    pomodoro.snoozed = false;
    state.pomodoro.set(pomodoro);
    reset_timer();
    *state::TASK_INPUT.write() = original_task_input;
    alert.is_shown = false;
    state.alert.set(alert);
    start_timer();
    update_task_history(settings.days_to_show);
}
