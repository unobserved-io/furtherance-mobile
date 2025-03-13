mod models {
    pub mod fur_settings;
    pub mod fur_task;
    pub mod fur_task_group;
}
mod helpers {
    pub mod actions;
    pub mod database;
    pub mod formatters;
    pub mod tasks;
    pub mod view_enums;
    pub mod views {
        pub mod task_input;
        pub mod timer;
    }
}
mod constants;
mod localization;
mod state;

use chrono::NaiveDate;
use constants::{FAVICON, MAIN_CSS};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsPlayFill, BsStopFill},
    Icon,
};
use helpers::{actions, database::db_init, formatters, views::task_input::validate_task_input};
use localization::Localization;
use models::{fur_settings::from_settings, fur_task_group::FurTaskGroup};

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    state::use_task_history_provider();
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        // TopNavView {}
        div { id: "page-content",
            TimerView {}
            TaskInputView {}
            TaskHistoryView {}
        }
    }
}

#[component]
pub fn TopNavView() -> Element {
    // TODO: Redo this for a better way to protect the safe area
    rsx! {
        div { id: "navbar" }
    }
}

#[component]
pub fn TimerView() -> Element {
    rsx! {
        div { id: "timer",
            div {
                h1 { class: "timer-text", "{state::TIMER_TEXT}" }
            }
        }
    }
}

#[component]
pub fn TaskInputView() -> Element {
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
pub fn TaskHistoryView() -> Element {
    rsx! {
        div { id: "task-history",
            for (date , task_groups) in use_context::<state::TaskHistory>().sorted.read().iter().rev() {
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
        from_settings(|settings| settings.show_seconds.clone()),
    );
    let formatted_date = formatters::format_history_date(&date);
    let total_earnings_str = format!("${:.2}", total_earnings);

    rsx! {
        div { id: "history-title-row",
            p { class: "bold", "{formatted_date}" }
            if from_settings(|settings| settings.show_daily_time_total) {
                div { class: "daily-totals",
                    p { class: "bold", "{total_time_str}" }
                    if from_settings(|settings| settings.show_task_earnings) && total_earnings > 0.0 {
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
        from_settings(|settings| settings.show_seconds),
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
                if from_settings(|settings| settings.show_task_project) {
                    p { class: "task-details", "{task_group.project}" }
                }
                if from_settings(|settings| settings.show_task_tags) {
                    p { class: "task-details", "{task_group.tags}" }
                }
            }

            div { class: "task-bubble-right",
                p { class: "bold", "{total_time_str}" }
                if from_settings(|settings| settings.show_task_earnings) && task_group.rate > 0.0 {
                    p { "{total_earnings_str}" }
                }
            }
        }
    }
}
