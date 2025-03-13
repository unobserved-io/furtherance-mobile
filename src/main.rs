mod models {
    pub mod fur_settings;
    pub mod fur_task;
    pub mod fur_task_group;
}
mod helpers {
    pub mod database;
    pub mod formatters;
    pub mod tasks;
    pub mod view_enums;
}
mod constants;
mod localization;

use std::collections::BTreeMap;

use chrono::NaiveDate;
use constants::{FAVICON, MAIN_CSS};
use dioxus::prelude::*;
use helpers::{database::db_init, formatters, tasks};
use models::{fur_settings::from_settings, fur_task_group::FurTaskGroup};

#[derive(Debug, Clone, Copy)]
struct TaskHistory {
    sorted: Signal<BTreeMap<NaiveDate, Vec<FurTaskGroup>>>,
}

fn use_task_history_provider() {
    let sorted = use_signal(|| tasks::get_task_history(365));
    use_context_provider(|| TaskHistory { sorted });
}

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_task_history_provider();
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        // TopNavView {}
        div {
            id: "page-content",
            TimerView {}
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
        div {
            id: "timer",
            div {
                h1 { class: "timer-text", "0:00:00" }
            }
        }
    }
}

#[component]
pub fn TaskHistoryView() -> Element {
    rsx! {
        div {
            id: "task-history",
            for (date, task_groups) in use_context::<TaskHistory>().sorted.read().iter().rev() {
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
        div {
            id: "history-title-row",
            p {
                class: "bold",
                "{formatted_date}"
            }
            if from_settings(|settings| settings.show_daily_time_total) {
                div {
                    class: "daily-totals",
                    p {
                        class: "bold",
                        "{total_time_str}"
                    }
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
        div {
            class: "task-bubble",
            if number_of_tasks > 1 {
                div {
                    class: "circle-number",
                    "{number_of_tasks}"
                }
            }

            div {
                class: "task-bubble-middle",
                p { class: "bold", "{task_group.name}"}
                if from_settings(|settings| settings.show_task_project) {
                    p { class: "task-details", "{task_group.project}"}
                }
                if from_settings(|settings| settings.show_task_tags) {
                    p { class: "task-details", "{task_group.tags}"}
                }
            }

            div {
                class: "task-bubble-right",
                p { class: "bold", "{total_time_str}" }
                if from_settings(|settings| settings.show_task_earnings) && task_group.rate > 0.0 {
                    p { "{total_earnings_str}" }
                }
            }
        }
    }
}
