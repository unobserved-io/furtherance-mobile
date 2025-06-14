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
    prelude::{spawn, spawn_forever},
    signals::Readable,
};
use std::sync::Once;

use crate::{
    database, formatters,
    helpers::{
        server::sync::{request_sync, sync_after_change},
        views::task_history::update_task_history,
    },
    loc,
    localization::Localization,
    models::{
        fur_persist::{reset_persisting_timer, FurPersist},
        fur_task::FurTask,
    },
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

                    let settings = state::SETTINGS.cloned();
                    let mut alert = state::ALERT.cloned();
                    let pomodoro = state::POMODORO.cloned();

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
                                *state::ALERT.write() = alert;
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
                                *state::ALERT.write() = alert;
                            }
                        }
                        continue;
                    }
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

    reset_timer();
}

pub fn start_timer() {
    *state::TIMER_START_TIME.write() = Local::now();
    *state::TIMER_IS_RUNNING.write() = true;
    let settings = state::SETTINGS.cloned();
    let mut pomodoro = state::POMODORO.cloned();

    if settings.pomodoro && !pomodoro.on_break {
        pomodoro.sessions += 1;
        *state::POMODORO.write() = pomodoro;
    }

    ensure_timer_running();

    if let Err(e) = database::persistence::update_persisting_timer(&FurPersist {
        is_running: true,
        task_input: state::TASK_INPUT.cloned(),
        start_time: state::TIMER_START_TIME.cloned(),
    }) {
        eprintln!("Error updating persisting timer: {}", e);
    }
}

pub fn reset_timer() {
    *state::TASK_INPUT.write() = String::new();
    *state::TIMER_TEXT.write() = get_timer_text(0);
    reset_persisting_timer();
}

pub fn get_timer_text(seconds_elapsed: i64) -> String {
    if state::TIMER_IS_RUNNING.cloned() {
        get_running_timer_text(seconds_elapsed)
    } else {
        get_stopped_timer_text()
    }
}

fn get_running_timer_text(seconds_elapsed: i64) -> String {
    let settings = state::SETTINGS.cloned();
    let pomodoro = state::POMODORO.cloned();

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
    let settings = state::SETTINGS.cloned();
    let pomodoro = state::POMODORO.cloned();

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
    let settings = state::SETTINGS.cloned();
    let mut alert = state::ALERT.cloned();
    let mut pomodoro = state::POMODORO.cloned();

    pomodoro.snoozed = false;
    stop_timer(Local::now());
    alert.is_shown = false;
    *state::ALERT.write() = alert;
    pomodoro.sessions = 0;
    *state::POMODORO.write() = pomodoro;
    update_task_history(settings.days_to_show);
    spawn_forever(async move {
        request_sync();
    });
}

fn start_break() {
    let settings = state::SETTINGS.cloned();
    let mut alert = state::ALERT.cloned();
    let mut pomodoro = state::POMODORO.cloned();

    let original_task_input = state::TASK_INPUT.cloned();
    pomodoro.on_break = true;
    pomodoro.snoozed = false;
    *state::POMODORO.write() = pomodoro;
    stop_timer(Local::now());
    *state::TASK_INPUT.write() = original_task_input;
    alert.is_shown = false;
    *state::ALERT.write() = alert;
    start_timer();
    update_task_history(settings.days_to_show);
    spawn_forever(async move {
        request_sync();
    });
}

fn stop_after_break() {
    let settings = state::SETTINGS.cloned();
    let mut alert = state::ALERT.cloned();
    let mut pomodoro = state::POMODORO.cloned();
    *state::TIMER_IS_RUNNING.write() = false;
    pomodoro.on_break = false;
    pomodoro.snoozed = false;
    pomodoro.sessions = 0;
    *state::POMODORO.write() = pomodoro;
    reset_timer();
    alert.is_shown = false;
    *state::ALERT.write() = alert;
    update_task_history(settings.days_to_show);
}

fn continue_after_break() {
    let settings = state::SETTINGS.cloned();
    let mut alert = state::ALERT.cloned();
    let mut pomodoro = state::POMODORO.cloned();

    *state::TIMER_IS_RUNNING.write() = false;
    let original_task_input = state::TASK_INPUT.cloned();
    pomodoro.on_break = false;
    pomodoro.snoozed = false;
    *state::POMODORO.write() = pomodoro;
    reset_timer();
    *state::TASK_INPUT.write() = original_task_input;
    alert.is_shown = false;
    *state::ALERT.write() = alert;
    start_timer();
    update_task_history(settings.days_to_show);
    sync_after_change();
}
