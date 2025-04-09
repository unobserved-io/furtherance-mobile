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

use std::collections::HashMap;

use chrono::{
    offset::LocalResult, DateTime, Duration, Local, MappedLocalTime, NaiveDate, NaiveDateTime,
    TimeZone,
};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsPlayFill, BsPlus, BsStopFill, BsX},
    Icon,
};
use fluent::FluentValue;

use crate::state;
use crate::{
    constants::SHEET_CSS,
    helpers::{
        actions, formatters,
        views::{task_input::validate_task_input, timer::get_stopped_timer_text},
    },
};
use crate::{database, loc, models::fur_task::FurTask};
use crate::{
    helpers::formatters::seconds_to_formatted_duration, models::fur_task_group::FurTaskGroup,
};
use crate::{helpers::views::task_history, localization::Localization};

static TIME_FORMAT: &str = "%Y-%m-%dT%H:%M";

#[component]
pub fn TimerView() -> Element {
    let sheets = use_context::<state::FurState>().sheets.read().clone();

    // Show pomodoro starting time if timer is not running
    // Must be async to prevent possible infinite loop
    spawn(async {
        if !state::TIMER_IS_RUNNING.cloned() {
            *state::TIMER_TEXT.write() = get_stopped_timer_text();
        }
    });

    rsx! {
        document::Stylesheet { href: SHEET_CSS }

        AddNewTask {}
        Timer {}
        TaskInput {}
        TaskHistory {}

        div { class: if sheets.new_task_is_shown || sheets.group_details_sheet.is_some() { "overlay visible" } else { "overlay" },
            ""
        }
        div { class: if sheets.new_task_is_shown { "sheet visible" } else { "sheet" }, NewTaskSheet {} }
        div { class: if sheets.group_details_sheet.is_some() && sheets.task_edit_sheet.is_none() { "sheet visible" } else { "sheet" },
            GroupDetailsSheet { task_group: sheets.group_details_sheet.clone() }
        }
    }
}

#[component]
pub fn AddNewTask() -> Element {
    rsx! {
        div { id: "add-new-task",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_task_is_shown = true;
                    state.sheets.set(new_sheets);
                },
                Icon { icon: BsPlus, width: 40, height: 40 }
            }
        }
    }
}

#[component]
pub fn Timer() -> Element {
    rsx! {
        div { id: "timer",
            div {
                h1 { class: "timer-text", "{state::TIMER_TEXT}" }
            }
        }
    }
}

#[component]
pub fn TaskInput() -> Element {
    rsx! {
        form {
            class: "task-input-form",
            onsubmit: move |event| {
                if state::TASK_INPUT.read().trim().is_empty() {
                    event.prevent_default();
                } else {
                    actions::start_stop_pressed();
                }
            },
            input {
                value: "{state::TASK_INPUT}",
                oninput: move |event| {
                    let new_value = validate_task_input(event.value());
                    *state::TASK_INPUT.write() = new_value;
                },
                placeholder: loc!("task-input-placeholder"),
            }
            button { r#type: "submit", class: "start-stop-button",
                if state::TIMER_IS_RUNNING.cloned() {
                    Icon { icon: BsStopFill, width: 25, height: 25 }
                } else {
                    Icon { icon: BsPlayFill, width: 25, height: 25 }
                }
            }
        }
    }
}

#[component]
pub fn TaskHistory() -> Element {
    rsx! {
        div { id: "task-history",
            for (date , task_groups) in use_context::<state::FurState>().tasks.read().iter().rev() {
                HistoryTitleRow { date: date.clone(), task_groups: task_groups.clone() }
                for task_group in task_groups {
                    HistoryGroupContainer { task_group: task_group.clone() }
                }
            }
        }
    }
}

#[component]
pub fn HistoryTitleRow(date: NaiveDate, task_groups: Vec<FurTaskGroup>) -> Element {
    let (total_time, total_earnings) = task_groups.iter().fold(
        (0i64, 0f32),
        |(accumulated_time, accumulated_earnings), group| {
            let group_time = group.total_time;
            let group_earnings = (group_time as f32 / 3600.0) * group.rate;

            (
                accumulated_time + group_time,
                accumulated_earnings + group_earnings,
            )
        },
    );
    let total_time_str = formatters::seconds_to_formatted_duration(total_time);
    let formatted_date = formatters::format_title_date(&date);
    let total_earnings_str = format!("${:.2}", total_earnings);

    rsx! {
        div { id: "history-title-row",
            p { class: "bold", "{formatted_date}" }
            if use_context::<state::FurState>().settings.read().show_daily_time_total {
                div { class: "daily-totals",
                    p { class: "bold", "{total_time_str}" }
                    if use_context::<state::FurState>().settings.read().show_task_earnings
                        && total_earnings > 0.0
                    {
                        p { "{total_earnings_str}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HistoryGroupContainer(task_group: FurTaskGroup) -> Element {
    let number_of_tasks = task_group.tasks.len();
    let total_time_str = formatters::seconds_to_formatted_duration(task_group.total_time);
    let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
    let total_earnings_str = format!("${:.2}", total_earnings);

    rsx! {
        div {
            class: "task-bubble",
            onclick: move |_| {
                let mut new_sheet = use_context::<state::FurState>().sheets.cloned();
                new_sheet.group_details_sheet = Some(task_group.clone());
                use_context::<state::FurState>().sheets.set(new_sheet);
            },

            if number_of_tasks > 1 {
                div { class: "circle-number", "{number_of_tasks}" }
            }

            div { class: "task-bubble-middle",
                p { class: "bold", "{task_group.name}" }
                if use_context::<state::FurState>().settings.read().show_task_project
                    && !task_group.project.is_empty()
                {
                    p { class: "task-details", "@{task_group.project}" }
                }
                if use_context::<state::FurState>().settings.read().show_task_tags
                    && !task_group.tags.is_empty()
                {
                    p { class: "task-details", "#{task_group.tags}" }
                }
            }

            div { class: "task-bubble-right",
                p { class: "bold", "{total_time_str}" }
                if use_context::<state::FurState>().settings.read().show_task_earnings
                    && task_group.rate > 0.0
                {
                    p { "{total_earnings_str}" }
                }
            }
        }
    }
}

#[component]
fn NewTaskSheet() -> Element {
    let mut task_input = use_signal(|| String::new());
    let one_hour_ago = Local::now() - Duration::hours(1);
    let mut start_time = use_signal(|| one_hour_ago.format(TIME_FORMAT).to_string());
    let mut stop_time = use_signal(|| Local::now().format(TIME_FORMAT).to_string());
    let save_text = loc!("save");
    let cancel_text = loc!("cancel");
    let start_colon = loc!("start-colon");
    let stop_colon = loc!("stop-colon");

    rsx! {
        div { class: "sheet-contents",
            h2 { "New Task" }
            input {
                class: "sheet-task-input",
                value: "{task_input}",
                oninput: move |event| {
                    let new_value = validate_task_input(event.value());
                    task_input.set(new_value);
                },
                placeholder: loc!("task-input-placeholder"),
            }

            // TODO: Test min/max on device (may work on device but not in simulator)
            br {}
            label { class: "sheet-label", "{start_colon}" }
            input {
                class: "sheet-task-datetime",
                r#type: "datetime-local",
                oninput: move |event| {
                    if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                        &event.value(),
                    ) {
                        if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                            &stop_time.cloned(),
                        ) {
                            if parsed_start_time < parsed_stop_time {
                                start_time.set(event.value())
                            } else {
                                start_time.set(start_time.cloned());
                            }
                        }
                    }
                },
                value: "{start_time}",
                max: "{stop_time}", // Seems unsupported by iOS
            }
            br {}
            label { class: "sheet-label", "{stop_colon}" }
            input {
                class: "sheet-task-datetime",
                r#type: "datetime-local",
                oninput: move |event| {
                    if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                        &event.value(),
                    ) {
                        if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                            &stop_time.cloned(),
                        ) {
                            if parsed_start_time > parsed_stop_time {
                                stop_time.set(event.value())
                            } else {
                                stop_time.set(stop_time.cloned());
                            }
                        }
                    }
                },
                value: "{stop_time}",
                min: "{start_time}", // Seems unsupported by iOS
            }
            br {}

            button {
                class: "sheet-cancel-button",
                onclick: move |_| {
                    task_input.set(String::new());
                    start_time.set(one_hour_ago.format(TIME_FORMAT).to_string());
                    stop_time.set(Local::now().format(TIME_FORMAT).to_string());
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_task_is_shown = false;
                    state.sheets.set(new_sheets);
                },
                "{cancel_text}"
            }
            button {
                class: "sheet-primary-button",
                onclick: move |event| {
                    if task_input.read().trim().is_empty() {
                        event.prevent_default();
                    } else {
                        if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                            &start_time.cloned(),
                        ) {
                            if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                                &stop_time.cloned(),
                            ) {
                                let (name, project, tags, rate) = formatters::split_task_input(
                                    &task_input.cloned(),
                                );
                                database::tasks::insert_task(
                                        &FurTask::new(
                                            name,
                                            parsed_start_time,
                                            parsed_stop_time,
                                            tags,
                                            project,
                                            rate,
                                            String::new(),
                                        ),
                                    )
                                    .expect("Couldn't write task to database.");
                                task_input.set(String::new());
                                start_time.set(one_hour_ago.format(TIME_FORMAT).to_string());
                                stop_time.set(Local::now().format(TIME_FORMAT).to_string());
                                let mut state = use_context::<state::FurState>();
                                let mut new_sheets = state.sheets.read().clone();
                                new_sheets.new_task_is_shown = false;
                                state.sheets.set(new_sheets);
                                task_history::update_task_history(
                                    use_context::<state::FurState>().settings.read().days_to_show,
                                );
                            }
                        }
                    }
                },
                "{save_text}"
            }
        }
    }
}

#[component]
fn GroupDetailsSheet(task_group: Option<FurTaskGroup>) -> Element {
    let mut task_input = use_signal(|| String::new());
    let one_hour_ago = Local::now() - Duration::hours(1);
    let mut start_time = use_signal(|| one_hour_ago.format(TIME_FORMAT).to_string());
    let mut stop_time = use_signal(|| Local::now().format(TIME_FORMAT).to_string());
    let rate_string = if let Some(group) = &task_group {
        format!("${:.2}", group.rate)
    } else {
        "$0.00".to_string()
    };
    let save_text = loc!("save");
    let cancel_text = loc!("cancel");
    let start_colon = loc!("start-colon");
    let stop_colon = loc!("stop-colon");

    rsx! {
        if let Some(group) = task_group {
            div { class: "sheet-contents",

                div { id: "group-buttons-row",
                    p {}
                    button {
                        class: "close-sheet-button",
                        onclick: move |_| {
                            let mut state = use_context::<state::FurState>();
                            let mut new_sheets = state.sheets.read().clone();
                            new_sheets.group_details_sheet = None;
                            state.sheets.set(new_sheets);
                        },
                        Icon { icon: BsX, width: 40, height: 40 }
                    }
                }

                h2 { "{group.name}" }
                if !group.project.is_empty() {
                    p { "@{group.project}" }
                }
                if !group.tags.is_empty() {
                    p { "#{group.tags}" }
                }
                if group.rate > 0.0 {
                    p { "{rate_string}" }
                }

                for task in group.tasks {
                    div {
                        class: "edit-task-bubble",
                        onclick: move |_| {
                            let mut new_sheet = use_context::<state::FurState>().sheets.cloned();
                            new_sheet.task_edit_sheet = Some(task.clone());
                            use_context::<state::FurState>().sheets.set(new_sheet);
                        },

                        p { class: "bold", "{get_start_to_stop_string(&task)}" }
                        p { "{get_total_task_time_string(&task)}" }
                    }
                }
            }
        } else {
            div {}
        }
    }
}

fn parse_datetime_from_str(datetime_str: &str) -> MappedLocalTime<DateTime<Local>> {
    match NaiveDateTime::parse_from_str(datetime_str, TIME_FORMAT) {
        Ok(naive) => Local.from_local_datetime(&naive),
        Err(_) => LocalResult::None,
    }
}

fn get_start_to_stop_string(task: &FurTask) -> String {
    loc!(
        "start-to-stop",
        &HashMap::from([
            (
                "start",
                FluentValue::from(task.start_time.format("%H:%M").to_string())
            ),
            (
                "stop",
                FluentValue::from(task.stop_time.format("%H:%M").to_string())
            )
        ])
    )
}

fn get_total_task_time_string(task: &FurTask) -> String {
    loc!(
        "total-time-dynamic",
        &HashMap::from([(
            "time",
            FluentValue::from(seconds_to_formatted_duration(task.total_time_in_seconds()))
        )])
    )
}
