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
    icons::bs_icons::{BsCheckSquare, BsPlayFill, BsSquare},
    Icon,
};

use crate::{constants::TODO_CSS, state};
use crate::{database, helpers::actions};
use crate::{
    helpers::{self, formatters},
    models::fur_todo::FurTodo,
};
use crate::{models::fur_settings::from_settings, state::TIMER_IS_RUNNING};

static CHECK_BOX_SIZE: u32 = 14;

#[component]
pub fn TodosView() -> Element {
    rsx! {
        document::Stylesheet { href: TODO_CSS }

        div { id: "todo-list",
            for (date , todos) in use_context::<state::AllTodos>().sorted.read().iter().rev() {
                TodoTitleRow { date: date.clone() }
                for todo in todos {
                    TodoListItem { todo: todo.clone() }
                }
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
                            Ok(_) => helpers::todos::update_all_todos(),
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

                    if from_settings(|settings| settings.show_todo_project) && !todo.project.is_empty() {
                        "  @{todo.project}"
                    }

                    if from_settings(|settings| settings.show_todo_tags) && !todo.tags.is_empty() {
                        "  #{todo.tags}"
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
