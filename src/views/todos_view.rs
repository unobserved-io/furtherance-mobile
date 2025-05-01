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

use chrono::{offset::LocalResult, Local, NaiveDate, TimeZone};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsCheckSquare, BsPlayFill, BsPlus, BsSquare, BsTrash3},
    Icon,
};

use crate::{
    constants::TODO_CSS,
    helpers::{server::sync::sync_after_change, views::todos::update_all_todos},
    loc,
    localization::Localization,
    state::{self, TODO_ID_TO_DELETE},
};
use crate::{database, helpers::actions};
use crate::{helpers::views::task_input::validate_task_input, state::TIMER_IS_RUNNING};
use crate::{
    helpers::{self, formatters},
    models::fur_todo::FurTodo,
};

static CHECK_BOX_SIZE: u32 = 14;

#[component]
pub fn TodosView() -> Element {
    let sheets = state::SHEETS.cloned();

    rsx! {

        document::Stylesheet { href: TODO_CSS }

        AddNewTodo {}

        div { id: "todo-list",
            for (date , todos) in state::TODOS.read().iter().rev() {
                TodoTitleRow { date: date.clone() }
                for todo in todos {
                    TodoListItem { todo: todo.clone() }
                }
            }
        }

        div { class: if sheets.new_todo_is_shown || sheets.edit_todo_sheet.is_some() { "overlay visible" } else { "overlay" },
            ""
        }
        div { class: if sheets.new_todo_is_shown { "sheet visible" } else { "sheet" },
            if sheets.new_todo_is_shown {
                NewTodoSheet {}
            }
        }
        div { class: if sheets.edit_todo_sheet.is_some() { "sheet visible" } else { "sheet" },
            if sheets.edit_todo_sheet.is_some() {
                EditTodoSheet { todo: sheets.edit_todo_sheet.clone() }
            }
        }
    }
}

#[component]
pub fn AddNewTodo() -> Element {
    rsx! {
        div { class: "add-new-todo",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.new_todo_is_shown = true;
                    *state::SHEETS.write() = new_sheets;
                },
                Icon { icon: BsPlus, width: 40, height: 40 }
            }
        }
    }
}

#[component]
fn TodoTitleRow(date: NaiveDate) -> Element {
    let formatted_date = formatters::format_title_date(&date);
    rsx! {
        div { id: "todo-title-row",
            p { class: "bold", "{formatted_date}" }
        }
    }
}

#[component]
fn TodoListItem(todo: FurTodo) -> Element {
    let todo_uid = todo.uid.clone();
    let todo_clone_two = todo.clone();
    rsx! {
        div { id: "todo-item",
            div { class: "todo-checkbox",
                button {
                    class: "no-bg-button",
                    onclick: move |_| {
                        match database::todos::toggle_todo_completed(&todo_uid) {
                            Ok(_) => {
                                helpers::views::todos::update_all_todos();
                                sync_after_change();
                            }
                            Err(e) => eprintln!("Error updating todo: {}", e),
                        }
                    },
                    if todo.is_completed {
                        Icon {
                            icon: BsCheckSquare,
                            width: CHECK_BOX_SIZE,
                            height: CHECK_BOX_SIZE,
                        }
                    } else {
                        Icon {
                            icon: BsSquare,
                            width: CHECK_BOX_SIZE,
                            height: CHECK_BOX_SIZE,
                        }
                    }
                }
            }

            div {
                class: "todo-text",
                onclick: move |_| {
                    let mut new_sheet = state::SHEETS.cloned();
                    new_sheet.edit_todo_sheet = Some(todo_clone_two.clone());
                    *state::SHEETS.write() = new_sheet;
                },
                p { class: if todo.is_completed { "strikethrough" } else { "" },
                    span { class: "bold", "{todo.name}" }

                    if state::SETTINGS.read().show_todo_project && !todo.project.is_empty() {
                        "  @{todo.project}"
                    }

                    if state::SETTINGS.read().show_todo_tags && !todo.tags.is_empty() {
                        "  #{todo.tags}"
                    }

                    if state::SETTINGS.read().show_todo_rate && todo.rate > 0.0 {
                        "  ${todo.rate}"
                    }
                }
            }

            div { class: "todo-start",
                button {
                    class: "no-bg-button",
                    onclick: move |_| {
                        actions::start_timer_with_task(todo.to_string());
                    },
                    if !TIMER_IS_RUNNING.cloned() {
                        Icon { icon: BsPlayFill, width: 25, height: 25 }
                    }
                }
            }
        }
    }
}

#[component]
fn NewTodoSheet() -> Element {
    let mut task_input = use_signal(|| String::new());
    let mut date = use_signal(|| Local::now().date_naive().to_string());
    let save_text = loc!("save");
    let cancel_text = loc!("cancel");
    let date_colon = loc!("date-colon");
    let new_todo_text = loc!("new-todo");

    rsx! {
        div { class: "sheet-contents",
            h2 { "{new_todo_text}" }
            input {
                class: "sheet-task-input",
                value: "{task_input}",
                oninput: move |event| {
                    let new_value = validate_task_input(event.value());
                    task_input.set(new_value);
                },
                placeholder: loc!("task-input-placeholder"),
            }

            br {}
            label { class: "sheet-label", "{date_colon}" }
            input {
                class: "sheet-todo-date",
                r#type: "date",
                oninput: move |event| { date.set(event.value()) },
                value: "{date}",
            }

            br {}
            button {
                class: "sheet-cancel-button",
                onclick: move |_| {
                    task_input.set(String::new());
                    date.set(Local::now().date_naive().to_string());
                    let mut new_sheets = state::SHEETS.cloned();
                    new_sheets.new_todo_is_shown = false;
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
                        if let Ok(naive_date) = NaiveDate::parse_from_str(
                            &date.cloned(),
                            "%Y-%m-%d",
                        ) {
                            if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
                                if let LocalResult::Single(parsed_datetime) = Local
                                    .from_local_datetime(&naive_datetime)
                                {
                                    let (name, project, tags, rate) = formatters::split_task_input(
                                        &task_input.cloned(),
                                    );
                                    database::todos::insert_todo(
                                            &FurTodo::new(name, project, tags, rate, parsed_datetime),
                                        )
                                        .expect("Couldn't write task to database.");
                                    task_input.set(String::new());
                                    date.set(Local::now().date_naive().to_string());
                                    let mut new_sheets = state::SHEETS.cloned();
                                    new_sheets.new_todo_is_shown = false;
                                    *state::SHEETS.write() = new_sheets;
                                    update_all_todos();
                                    sync_after_change();
                                }
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
fn EditTodoSheet(todo: Option<FurTodo>) -> Element {
    if let Some(todo) = todo {
        let todo_clone = todo.clone();
        let todo_uid = todo.uid.clone();
        let mut todo_input = use_signal(|| todo.to_string());
        let mut date = use_signal(|| todo.date.date_naive().to_string());

        rsx! {
            div { class: "sheet-contents",
                div { id: "group-buttons-row",
                    button {
                        class: "no-bg-button",
                        onclick: move |_| {
                            fn delete_todo() {
                                if let Some(todo_id) = TODO_ID_TO_DELETE.cloned() {
                                    if let Err(e) = database::todos::delete_todo_by_id(&todo_id) {
                                        eprintln!("Failed to delete todo: {}", e);
                                    }
                                }
                                let mut alert = state::ALERT.cloned();
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.edit_todo_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                                *TODO_ID_TO_DELETE.write() = None;
                                alert.close();
                                *state::ALERT.write() = alert.clone();
                                update_all_todos();
                                sync_after_change();
                            }
                            fn close_alert() {
                                let mut alert = state::ALERT.cloned();
                                *TODO_ID_TO_DELETE.write() = None;
                                alert.close();
                                *state::ALERT.write() = alert.clone();
                            }
                            let mut alert = state::ALERT.cloned();
                            let settings = state::SETTINGS.cloned();
                            if settings.show_delete_confirmation {
                                *TODO_ID_TO_DELETE.write() = Some(todo_uid.clone());
                                alert.is_shown = true;
                                alert.title = loc!("delete-todo-question");
                                alert.message = loc!("delete-todo-description");
                                alert.confirm_button = (loc!("delete"), || delete_todo());
                                alert.cancel_button = Some((loc!("cancel"), || close_alert()));
                                *state::ALERT.write() = alert.clone();
                            } else {
                                if let Err(e) = database::todos::delete_todo_by_id(&todo_uid) {
                                    eprintln!("Failed to delete todo: {}", e);
                                }
                                let mut new_sheets = state::SHEETS.cloned();
                                new_sheets.edit_todo_sheet = None;
                                *state::SHEETS.write() = new_sheets;
                                update_all_todos();
                                sync_after_change();
                            }
                        },
                        Icon { icon: BsTrash3, width: 25, height: 25 }
                    }
                }

                h2 { {loc!("edit-todo")} }
                input {
                    class: "sheet-task-input",
                    value: "{todo_input}",
                    oninput: move |event| {
                        let new_value = validate_task_input(event.value());
                        todo_input.set(new_value);
                    },
                    placeholder: loc!("task-input-placeholder"),
                }

                br {}
                label { class: "sheet-label", {loc!("date-colon")} }
                input {
                    class: "sheet-todo-date",
                    r#type: "date",
                    oninput: move |event| { date.set(event.value()) },
                    value: "{date}",
                }

                br {}
                button {
                    class: "sheet-cancel-button",
                    onclick: move |_| {
                        let mut new_sheets = state::SHEETS.cloned();
                        new_sheets.edit_todo_sheet = None;
                        *state::SHEETS.write() = new_sheets;
                    },
                    {loc!("cancel")}
                }
                button {
                    class: "sheet-primary-button",
                    onclick: move |event| {
                        if todo_input.read().trim().is_empty() {
                            event.prevent_default();
                        } else {
                            if let Ok(naive_date) = NaiveDate::parse_from_str(
                                &date.cloned(),
                                "%Y-%m-%d",
                            ) {
                                if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
                                    if let LocalResult::Single(parsed_datetime) = Local
                                        .from_local_datetime(&naive_datetime)
                                    {
                                        let (name, project, tags, rate) = formatters::split_task_input(
                                            &todo_input.cloned(),
                                        );
                                        let mut new_todo = todo_clone.clone();
                                        new_todo.name = name;
                                        new_todo.project = project;
                                        new_todo.tags = tags;
                                        new_todo.rate = rate;
                                        new_todo.date = parsed_datetime;
                                        if let Err(e) = database::todos::update_todo(&new_todo) {
                                            eprintln!("Error updating todo in database: {}", e);
                                        }
                                        let mut new_sheets = state::SHEETS.cloned();
                                        new_sheets.edit_todo_sheet = None;
                                        *state::SHEETS.write() = new_sheets;
                                        update_all_todos();
                                        sync_after_change();
                                    }
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
