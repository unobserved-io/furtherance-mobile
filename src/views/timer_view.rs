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
    icons::bs_icons::{BsPencil, BsPlayFill, BsPlusLg, BsStopFill, BsTrash3, BsXLg},
    Icon,
};
use fluent::FluentValue;

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
use crate::{
    helpers::server::sync::sync_after_change,
    state::{self, TASK_IDS_TO_DELETE},
};
use crate::{helpers::views::task_history, localization::Localization};

static DATE_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M";
static TIME_FORMAT: &str = "%H:%M";

#[component]
pub fn TimerView() -> Element {
    let sheets = state::SHEETS.cloned();

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

        div { class: if sheets.new_task_is_shown { "sheet visible" } else { "sheet" }, NewTaskSheet {} }
        div { class: if sheets.group_details_sheet.is_some() && sheets.task_edit_sheet.is_none()
    && sheets.add_to_group_sheet.is_none() && sheets.edit_group_sheet.is_none() { "sheet visible" } else { "sheet" },
            if sheets.group_details_sheet.is_some() && sheets.task_edit_sheet.is_none()
                && sheets.add_to_group_sheet.is_none() && sheets.edit_group_sheet.is_none()
            {
                GroupDetailsSheet { task_group: sheets.group_details_sheet.clone() }
            }
        }
        div { class: if sheets.task_edit_sheet.is_some() { "sheet visible" } else { "sheet" },
            if sheets.task_edit_sheet.is_some() {
                TaskEditSheet { task: sheets.task_edit_sheet.clone() }
            }
        }
        div { class: if sheets.add_to_group_sheet.is_some() { "sheet visible" } else { "sheet" },
            if sheets.add_to_group_sheet.is_some() {
                AddToGroupSheet { group: sheets.add_to_group_sheet.clone() }
            }
        }
        div { class: if sheets.edit_group_sheet.is_some() { "sheet visible" } else { "sheet" },
            if sheets.edit_group_sheet.is_some() {
                EditGroupSheet { group: sheets.edit_group_sheet.clone() }
            }
        }
    }
}

#[component]
pub fn AddNewTask() -> Element {
    rsx! {
        div { class: "add-new-task",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.new_task_is_shown = true;
                    *state::SHEETS.write() = new_sheets;
                },
                Icon { icon: BsPlusLg, width: 30, height: 30 }
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
                    let old_value = state::TASK_INPUT.cloned();

                    if state::TIMER_IS_RUNNING.cloned() {
                        if new_value.trim().is_empty() {
                            event.prevent_default();
                            *state::TASK_INPUT.write() = old_value.clone();
                        } else {
                            *state::TASK_INPUT.write() = new_value.clone();
                            if let Err(e) = database::persistence::update_persisting_timer_task_input(
                                &new_value,
                            ) {
                                eprintln!("Error updating persisting timer task input: {}", e);
                            }
                        }
                    } else {
                        *state::TASK_INPUT.write() = new_value.clone();
                    }
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
            for (date , task_groups) in state::TASKS.read().iter().rev() {
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
            if state::SETTINGS.read().show_daily_time_total {
                div { class: "daily-totals",
                    p { class: "bold", "{total_time_str}" }
                    if state::SETTINGS.read().show_task_earnings && total_earnings > 0.0 {
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
                let mut new_sheet = state::SHEETS.cloned();
                if number_of_tasks == 1 {
                    new_sheet.task_edit_sheet = Some(task_group.tasks.first().unwrap().clone());
                } else {
                    new_sheet.group_details_sheet = Some(task_group.clone());
                }
                *state::SHEETS.write() = new_sheet;
            },

            if number_of_tasks > 1 {
                div { class: "circle-number", "{number_of_tasks}" }
            }

            div { class: "task-bubble-middle",
                p { class: "bold", "{task_group.name}" }
                if state::SETTINGS.read().show_task_project && !task_group.project.is_empty() {
                    p { class: "task-details", "@{task_group.project}" }
                }
                if state::SETTINGS.read().show_task_tags && !task_group.tags.is_empty() {
                    p { class: "task-details", "#{task_group.tags}" }
                }
            }

            div { class: "task-bubble-right",
                p { class: "bold", "{total_time_str}" }
                if state::SETTINGS.read().show_task_earnings && task_group.rate > 0.0 {
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
    let mut start_time = use_signal(|| one_hour_ago.format(DATE_TIME_FORMAT).to_string());
    let mut stop_time = use_signal(|| Local::now().format(DATE_TIME_FORMAT).to_string());
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
                    if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                        &event.value(),
                    ) {
                        if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
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
                    start_time.set(one_hour_ago.format(DATE_TIME_FORMAT).to_string());
                    stop_time.set(Local::now().format(DATE_TIME_FORMAT).to_string());
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.new_task_is_shown = false;
                    *state::SHEETS.write() = new_sheets;
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
                                start_time.set(one_hour_ago.format(DATE_TIME_FORMAT).to_string());
                                stop_time.set(Local::now().format(DATE_TIME_FORMAT).to_string());
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.new_task_is_shown = false;
                                *state::SHEETS.write() = new_sheets;
                                task_history::update_task_history(
                                    state::SETTINGS.read().days_to_show,
                                );
                                sync_after_change();
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
    let task_group_clone = task_group.clone();
    let task_group_clone_two = task_group.clone();
    let task_group_clone_three = task_group.clone();
    let mut alert = state::ALERT.cloned();

    rsx! {
        if let Some(group) = task_group {
            div { class: "sheet-contents",

                div { id: "group-buttons-row",
                    div {
                        button {
                            class: "no-bg-button",
                            onclick: move |_| {
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.add_to_group_sheet = task_group_clone.clone();
                                *state::SHEETS.write() = new_sheets;
                            },
                            Icon { icon: BsPlusLg, width: 25, height: 25 }
                        }

                        button {
                            class: "no-bg-button",
                            onclick: move |_| {
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.edit_group_sheet = task_group_clone_two.clone();
                                *state::SHEETS.write() = new_sheets;
                            },
                            Icon { icon: BsPencil, width: 25, height: 25 }
                        }

                        button {
                            class: "no-bg-button",
                            onclick: move |_| {
                                fn delete_whole_group() {
                                    if let Some(task_ids) = TASK_IDS_TO_DELETE.cloned() {
                                        if let Err(e) = database::tasks::delete_tasks_by_ids(&task_ids) {
                                            eprintln!("Failed to delete tasks: {}", e);
                                        }
                                    }
                                    let mut alert = state::ALERT.cloned();
                                    let mut new_sheets = state::SHEETS.cloned();
                                    new_sheets.group_details_sheet = None;
                                    *state::SHEETS.write() = new_sheets;
                                    *TASK_IDS_TO_DELETE.write() = None;
                                    alert.close();
                                    *state::ALERT.write() = alert.clone();
                                    task_history::update_task_history(state::SETTINGS.read().days_to_show);
                                    sync_after_change();
                                }
                                fn close_alert() {
                                    let mut alert = state::ALERT.cloned();
                                    *TASK_IDS_TO_DELETE.write() = None;
                                    alert.close();
                                    *state::ALERT.write() = alert.clone();
                                }
                                if let Some(task_group) = task_group_clone_three.clone() {
                                    let settings = state::SETTINGS.cloned();
                                    if settings.show_delete_confirmation {
                                        *TASK_IDS_TO_DELETE.write() = Some(task_group.all_task_ids());
                                        alert.is_shown = true;
                                        alert.title = loc!("delete-all-question");
                                        alert.message = loc!("delete-all-description");
                                        alert.confirm_button = (loc!("delete-all"), || delete_whole_group());
                                        alert.cancel_button = Some((loc!("cancel"), || close_alert()));
                                        *state::ALERT.write() = alert.clone();
                                    } else {
                                        if let Err(e) = database::tasks::delete_tasks_by_ids(
                                            &task_group.all_task_ids(),
                                        ) {
                                            eprintln!("Failed to delete tasks: {}", e);
                                        }
                                        let mut new_sheets = state::SHEETS.cloned();
                                        new_sheets.group_details_sheet = None;
                                        *state::SHEETS.write() = new_sheets;
                                        task_history::update_task_history(state::SETTINGS.read().days_to_show);
                                        sync_after_change();
                                    }
                                }
                            },
                            Icon { icon: BsTrash3, width: 25, height: 25 }
                        }
                    }
                    div {
                        button {
                            class: "close-sheet-button",
                            onclick: move |_| {
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.group_details_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                            },
                            Icon { icon: BsXLg, width: 25, height: 25 }
                        }
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
                    p { {format!("${:.2}", group.rate)} }
                }

                for task in group.tasks {
                    div {
                        class: "edit-task-bubble",
                        onclick: move |_| {
                            let mut new_sheet = state::SHEETS.cloned();
                            new_sheet.task_edit_sheet = Some(task.clone());
                            *state::SHEETS.write() = new_sheet;
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

#[component]
fn TaskEditSheet(task: Option<FurTask>) -> Element {
    if let Some(task) = task {
        let mut task_input = use_signal(|| task.to_string());
        let mut start_time = use_signal(|| task.start_time.format(DATE_TIME_FORMAT).to_string());
        let mut stop_time = use_signal(|| task.stop_time.format(DATE_TIME_FORMAT).to_string());

        let task_uid = task.uid.clone();
        let task_uid_vec: Vec<String> = vec![task.uid.clone()];
        let task_currency = task.currency.clone();

        rsx! {
            div { class: "sheet-contents",
                div { id: "group-buttons-row",
                    button {
                        class: "no-bg-button",
                        onclick: move |_| {
                            fn delete_task() {
                                if let Some(tasks_id) = TASK_IDS_TO_DELETE.cloned() {
                                    if let Err(e) = database::tasks::delete_tasks_by_ids(&tasks_id) {
                                        eprintln!("Failed to delete task: {}", e);
                                    }
                                }
                                let mut alert = state::ALERT.cloned();
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.task_edit_sheet = None;
                                new_sheets.group_details_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                                *TASK_IDS_TO_DELETE.write() = None;
                                alert.close();
                                *state::ALERT.write() = alert.clone();
                                task_history::update_task_history(state::SETTINGS.read().days_to_show);
                                sync_after_change();
                            }
                            fn close_alert() {
                                let mut alert = state::ALERT.cloned();
                                *TASK_IDS_TO_DELETE.write() = None;
                                alert.close();
                                *state::ALERT.write() = alert.clone();
                            }
                            let mut alert = state::ALERT.cloned();
                            let settings = state::SETTINGS.cloned();
                            if settings.show_delete_confirmation {
                                *TASK_IDS_TO_DELETE.write() = Some(task_uid_vec.clone());
                                alert.is_shown = true;
                                alert.title = loc!("delete-task-question");
                                alert.message = loc!("delete-task-description");
                                alert.confirm_button = (loc!("delete"), || delete_task());
                                alert.cancel_button = Some((loc!("cancel"), || close_alert()));
                                *state::ALERT.write() = alert.clone();
                            } else {
                                if let Err(e) = database::tasks::delete_tasks_by_ids(&task_uid_vec) {
                                    eprintln!("Failed to delete task: {}", e);
                                }
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.task_edit_sheet = None;
                                new_sheets.group_details_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                                task_history::update_task_history(state::SETTINGS.read().days_to_show);
                                sync_after_change();
                            }
                        },
                        Icon { icon: BsTrash3, width: 25, height: 25 }
                    }
                }

                h2 { {loc!("edit-task")} }
                input {
                    class: "sheet-task-input",
                    value: "{task_input}",
                    oninput: move |event| {
                        let new_value = validate_task_input(event.value());
                        task_input.set(new_value);
                    },
                    placeholder: task.to_string(),
                }

                // TODO: Test min/max on device (may work on device but not in simulator)
                br {}
                label { class: "sheet-label", {loc!("start-colon")} }
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
                label { class: "sheet-label", {loc!("stop-colon")} }
                input {
                    class: "sheet-task-datetime",
                    r#type: "datetime-local",
                    oninput: move |event| {
                        if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                            &event.value(),
                        ) {
                            if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
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
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.task_edit_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                    },
                    {loc!("cancel")}
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
                                    if name != task.name || project != task.project || tags != task.tags
                                        || rate != task.rate || parsed_start_time != task.start_time
                                        || parsed_stop_time != task.stop_time
                                    {
                                        database::tasks::update_task(
                                                &FurTask {
                                                    name,
                                                    start_time: parsed_start_time,
                                                    stop_time: parsed_stop_time,
                                                    tags,
                                                    project,
                                                    rate,
                                                    currency: task_currency.clone(),
                                                    uid: task_uid.clone(),
                                                    is_deleted: task.is_deleted,
                                                    last_updated: chrono::Utc::now().timestamp(),
                                                },
                                            )
                                            .expect("Couldn't update task in database.");
                                    }
                                    task_history::update_task_history(
                                        state::SETTINGS.read().days_to_show,
                                    );
                                    sync_after_change();
                                    let mut new_sheets = state::SHEETS.cloned();
                                    new_sheets.group_details_sheet = None;
                                    new_sheets.task_edit_sheet = None;
                                    *state::SHEETS.write() = new_sheets;
                                }
                            }
                        }
                    },
                    {loc!("save")}
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn AddToGroupSheet(group: Option<FurTaskGroup>) -> Element {
    if let Some(group) = group {
        let one_hour_ago = Local::now() - Duration::hours(1);
        let mut start_time = use_signal(|| one_hour_ago.format(TIME_FORMAT).to_string());
        let mut stop_time = use_signal(|| Local::now().format(TIME_FORMAT).to_string());
        let task_date_time = if let Some(task) = group.tasks.first() {
            task.start_time
        } else {
            Local::now()
        };
        let date_string = task_date_time.format("%h %e").to_string();
        let month_day_year_t = use_signal(|| task_date_time.format("%Y-%m-%dT").to_string());

        rsx! {
            div { class: "sheet-contents",
                h2 { {loc!("add-to-group")} }
                input {
                    class: "sheet-task-input",
                    value: "{group.to_string()}",
                    placeholder: group.to_string(),
                    disabled: true,
                }

                // TODO: Test min/max on device (may work on device but not in simulator)
                br {}
                label { class: "sheet-label", {loc!("start-colon") + " " + { &date_string }} }
                input {
                    class: "sheet-task-datetime",
                    r#type: "time",
                    oninput: move |event| {
                        if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                            &(month_day_year_t.cloned() + &event.value()),
                        ) {
                            if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                                &(month_day_year_t.cloned() + &stop_time.cloned()),
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
                label { class: "sheet-label", {loc!("stop-colon") + " " + { &date_string }} }
                input {
                    class: "sheet-task-datetime",
                    r#type: "time",
                    oninput: move |event| {
                        if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                            &(month_day_year_t.cloned() + &event.value()),
                        ) {
                            if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                                &(month_day_year_t.cloned() + &start_time.cloned()),
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
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.add_to_group_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                    },
                    {loc!("cancel")}
                }
                button {
                    class: "sheet-primary-button",
                    onclick: move |_| {
                        if let MappedLocalTime::Single(parsed_start_time) = parse_datetime_from_str(
                            &(month_day_year_t.cloned() + &start_time.cloned()),
                        ) {
                            if let MappedLocalTime::Single(parsed_stop_time) = parse_datetime_from_str(
                                &(month_day_year_t.cloned() + &stop_time.cloned()),
                            ) {
                                let (name, project, tags, rate) = formatters::split_task_input(
                                    &group.to_string(),
                                );
                                if let Err(e) = database::tasks::insert_task(
                                    &FurTask::new(
                                        name,
                                        parsed_start_time,
                                        parsed_stop_time,
                                        tags,
                                        project,
                                        rate,
                                        String::new(),
                                    ),
                                ) {
                                    eprintln!("Error inserting task into database: {}", e);
                                }
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.group_details_sheet = None;
                                new_sheets.add_to_group_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                                task_history::update_task_history(state::SETTINGS.read().days_to_show);
                                sync_after_change();
                            }
                        }
                    },
                    {loc!("save")}
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn EditGroupSheet(group: Option<FurTaskGroup>) -> Element {
    if let Some(group) = group {
        let group_clone = group.clone();
        let mut group_input = use_signal(|| group.to_string());

        rsx! {
            div { class: "sheet-contents",
                h2 { {loc!("edit-group")} }
                input {
                    class: "sheet-task-input",
                    value: "{group_input}",
                    oninput: move |event| {
                        let new_value = validate_task_input(event.value());
                        group_input.set(new_value);
                    },
                    placeholder: group.to_string(),
                }
                br {}

                button {
                    class: "sheet-cancel-button",
                    onclick: move |_| {
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.edit_group_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                    },
                    {loc!("cancel")}
                }
                button {
                    class: "sheet-primary-button",
                    onclick: move |_| {
                        let (name, project, tags, rate) = formatters::split_task_input(
                            &group_input.cloned(),
                        );
                        let mut new_group = group_clone.clone();
                        new_group.name = name;
                        new_group.project = project;
                        new_group.tags = tags;
                        new_group.rate = rate;
                        if let Err(e) = database::tasks::update_group_of_tasks(&new_group) {
                            eprintln!("Error updating task group in database: {}", e);
                        }
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.group_details_sheet = None;
                        new_sheets.edit_group_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                        task_history::update_task_history(state::SETTINGS.read().days_to_show);
                        sync_after_change();
                    },
                    {loc!("save")}
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn parse_datetime_from_str(datetime_str: &str) -> MappedLocalTime<DateTime<Local>> {
    match NaiveDateTime::parse_from_str(datetime_str, DATE_TIME_FORMAT) {
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
