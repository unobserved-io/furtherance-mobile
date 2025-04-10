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

use chrono::{
    format::ParseErrorKind, offset::LocalResult, DateTime, Local, NaiveDate, ParseError, TimeZone,
};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsCheckSquare, BsPlayFill, BsPlus, BsSquare},
    Icon,
};

use crate::{
    constants::TODO_CSS, helpers::views::todos::update_all_todos, loc, localization::Localization,
    state,
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
    let sheets = use_context::<state::FurState>().sheets.read().clone();

    rsx! {

        document::Stylesheet { href: TODO_CSS }

        AddNewTodo {}

        div { id: "todo-list",
            for (date , todos) in use_context::<state::FurState>().todos.read().iter().rev() {
                TodoTitleRow { date: date.clone() }
                for todo in todos {
                    TodoListItem { todo: todo.clone() }
                }
            }
        }

        div { class: if sheets.new_todo_is_shown { "overlay visible" } else { "overlay" }, "" }
        div { class: if sheets.new_todo_is_shown { "sheet visible" } else { "sheet" }, NewTodoSheet {} }
    }
}

#[component]
pub fn AddNewTodo() -> Element {
    rsx! {
        div { id: "add-new-todo",
            button {
                class: "no-bg-button",
                onclick: move |_| {
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_todo_is_shown = true;
                    state.sheets.set(new_sheets);
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
    let mut todo_clone = todo.clone();
    rsx! {
        div { id: "todo-item",
            div { class: "todo-checkbox",
                button {
                    class: "no-bg-button",
                    onclick: move |_| {
                        todo_clone.is_completed = !todo_clone.is_completed;
                        match database::todos::update_todo(&todo_clone) {
                            Ok(_) => helpers::views::todos::update_all_todos(),
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

            div { class: "todo-text",
                p { class: if todo.is_completed { "strikethrough" } else { "" },
                    span { class: "bold", "{todo.name}" }

                    if use_context::<state::FurState>().settings.read().show_todo_project
                        && !todo.project.is_empty()
                    {
                        "  @{todo.project}"
                    }

                    if use_context::<state::FurState>().settings.read().show_todo_tags
                        && !todo.tags.is_empty()
                    {
                        "  #{todo.tags}"
                    }

                    if use_context::<state::FurState>().settings.read().show_todo_rate && todo.rate > 0.0 {
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
                    let mut state = use_context::<state::FurState>();
                    let mut new_sheets = state.sheets.read().clone();
                    new_sheets.new_todo_is_shown = false;
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
                                    let mut state = use_context::<state::FurState>();
                                    let mut new_sheets = state.sheets.read().clone();
                                    new_sheets.new_todo_is_shown = false;
                                    state.sheets.set(new_sheets);
                                    update_all_todos();
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
