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

use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsPlayFill, BsStopFill},
    Icon,
};

use crate::helpers::{actions, formatters, views::task_input::validate_task_input};
use crate::loc;
use crate::localization::Localization;
use crate::models::fur_task_group::FurTaskGroup;
use crate::state;

#[component]
pub fn TimerView() -> Element {
    rsx! {
        Timer {}
        TaskInput {}
        TaskHistory {}
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
    let total_time_str = formatters::seconds_to_formatted_duration(
        total_time,
        use_context::<state::FurState>()
            .settings
            .read()
            .show_seconds,
    );
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
    let total_time_str = formatters::seconds_to_formatted_duration(
        task_group.total_time,
        use_context::<state::FurState>()
            .settings
            .read()
            .show_seconds,
    );
    let total_earnings = task_group.rate * (task_group.total_time as f32 / 3600.0);
    let total_earnings_str = format!("${:.2}", total_earnings);

    rsx! {
        div { class: "task-bubble",
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
