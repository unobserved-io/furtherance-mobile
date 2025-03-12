mod models {
    pub mod fur_settings;
    pub mod fur_task;
    pub mod fur_task_group;
}
mod helpers {
    pub mod database;
    pub mod view_enums;
}
mod constants;
mod localization;

use chrono::Local;
use constants::{FAVICON, MAIN_CSS};
use dioxus::prelude::*;
use helpers::database::db_init;
use models::fur_task::FurTask;

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        Timer {}
        TaskHistory {}
    }
}

#[component]
pub fn Timer() -> Element {
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
pub fn TaskHistory() -> Element {
    let task_history = use_signal(|| {
        vec![
            FurTask {
                name: String::from("First task"),
                start_time: Local::now(),
                stop_time: Local::now(),
                tags: String::from("one #two"),
                project: String::from("Proj"),
                rate: 10.0,
                currency: String::new(),
                uid: String::from("fjsdljfjwrkejlj"),
                is_deleted: false,
                last_updated: 7432483084028,
            },
            FurTask {
                name: String::from("First task"),
                start_time: Local::now(),
                stop_time: Local::now(),
                tags: String::from("one #two"),
                project: String::from("Proj"),
                rate: 10.0,
                currency: String::new(),
                uid: String::from("fjsdljfjwrkejlj"),
                is_deleted: false,
                last_updated: 7432483084028,
            },
            FurTask {
                name: String::from("Second task"),
                start_time: Local::now(),
                stop_time: Local::now(),
                tags: String::from("one #two"),
                project: String::from("Proj"),
                rate: 10.0,
                currency: String::new(),
                uid: String::from("fjsdljfjwrkejlj"),
                is_deleted: false,
                last_updated: 7432483084028,
            },
        ]
    });

    rsx! {
        div {
            id: "task-history",
            for task_group in task_history.iter() {
                div {
                    class: "task-bubble",
                    p { class: "task-name", "{task_group.name}"}
                    p { class: "task-details", "{task_group.project}"}
                    p { class: "task-details", "{task_group.tags}"}
                }
            }
        }
    }
}
