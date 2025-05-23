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

use chrono::Local;
use dioxus::{prelude::spawn_forever, signals::Readable};

use crate::{
    database::{
        self,
        tasks::{SortBy, SortOrder},
    },
    models::{fur_task::FurTask, fur_task_group::FurTaskGroup},
    state,
};

pub fn get_task_history(limit: i64) -> BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> {
    let mut grouped_tasks_by_date: BTreeMap<chrono::NaiveDate, Vec<FurTaskGroup>> = BTreeMap::new();

    match database::tasks::retrieve_tasks_with_day_limit(
        limit,
        SortBy::StopTime,
        SortOrder::Descending,
    ) {
        Ok(all_tasks) => {
            let tasks_by_date = group_tasks_by_date(all_tasks);

            for (date, tasks) in tasks_by_date {
                let mut all_groups: Vec<FurTaskGroup> = vec![];
                for task in tasks {
                    if let Some(matching_group) =
                        all_groups.iter_mut().find(|x| x.is_equal_to(&task))
                    {
                        matching_group.add(task);
                    } else {
                        all_groups.push(FurTaskGroup::new_from(task));
                    }
                }
                grouped_tasks_by_date.insert(date, all_groups);
            }
        }
        Err(e) => {
            eprintln!("Error retrieving tasks from database: {}", e);
        }
    }
    grouped_tasks_by_date
}

fn group_tasks_by_date(tasks: Vec<FurTask>) -> BTreeMap<chrono::NaiveDate, Vec<FurTask>> {
    let mut grouped_tasks: BTreeMap<chrono::NaiveDate, Vec<FurTask>> = BTreeMap::new();

    for task in tasks {
        let date = task.start_time.date_naive(); // Extract the date part
        grouped_tasks
            .entry(date)
            .or_insert_with(Vec::new)
            .push(task);
    }

    grouped_tasks
}

pub fn update_task_history(days_to_show: i64) {
    spawn_forever(async move {
        let task_history = get_task_history(days_to_show);
        *state::TASKS.write() = task_history.clone();
        update_todos_after_refresh(days_to_show);
    });
}

pub fn update_todos_after_refresh(days_to_show: i64) {
    let today = Local::now().date_naive();
    let task_history = get_task_history(days_to_show);
    let mut new_todos = state::TODOS.cloned();

    if let Some(todays_todos) = new_todos.get_mut(&today) {
        if let Some(todays_tasks) = task_history.get(&today) {
            for todo in todays_todos.iter_mut() {
                if let Some(_) = todays_tasks
                    .iter()
                    .find(|task_group| task_group.to_string() == todo.to_string())
                {
                    match database::todos::set_todo_completed(&todo.uid) {
                        Ok(_) => todo.is_completed = true,
                        Err(e) => {
                            eprintln!("Error while marking todo {} as completed: {}", todo.uid, e)
                        }
                    }
                }
            }

            *state::TODOS.write() = new_todos;
        }
    };
}
