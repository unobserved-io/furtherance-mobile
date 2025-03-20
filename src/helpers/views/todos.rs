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

use std::collections::BTreeMap;

use chrono::{Local, TimeDelta};
use dioxus::{hooks::use_context, signals::Writable};

use crate::{database, models::fur_todo::FurTodo, state};

pub fn update_all_todos() {
    use_context::<state::FurState>().todos.set(get_all_todos());
}

pub fn get_all_todos() -> BTreeMap<chrono::NaiveDate, Vec<FurTodo>> {
    let future_limit = Local::now() + TimeDelta::days(3);
    let past_limit = Local::now() - TimeDelta::days(60);
    let mut todos_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTodo>> = BTreeMap::new();

    match database::todos::retrieve_todos_between_dates(
        past_limit.to_string(),
        future_limit.to_string(),
    ) {
        Ok(all_todos) => {
            todos_by_date = group_todos_by_date(all_todos);
        }
        Err(e) => {
            eprintln!("Error retrieving todos from database: {}", e);
        }
    }

    todos_by_date
}

fn group_todos_by_date(todos: Vec<FurTodo>) -> BTreeMap<chrono::NaiveDate, Vec<FurTodo>> {
    let mut grouped_todos: BTreeMap<chrono::NaiveDate, Vec<FurTodo>> = BTreeMap::new();

    for todo in todos {
        let date = todo.date.date_naive();
        grouped_todos
            .entry(date)
            .or_insert_with(Vec::new)
            .push(todo);
    }

    grouped_todos
}
