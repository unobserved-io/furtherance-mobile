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

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FurTodo {
    pub name: String,
    pub project: String,
    pub tags: String,
    pub rate: f32,
    pub currency: String,
    pub date: DateTime<Local>,
    pub uid: String,
    pub is_completed: bool,
    pub is_deleted: bool,
    pub last_updated: i64,
}

impl FurTodo {
    pub fn new(
        name: String,
        project: String,
        tags: String,
        rate: f32,
        date: DateTime<Local>,
    ) -> Self {
        let uid = generate_todo_uid(&name, &date);

        FurTodo {
            name,
            project,
            tags,
            rate,
            currency: String::new(),
            date,
            uid,
            is_completed: false,
            is_deleted: false,
            last_updated: Utc::now().timestamp(),
        }
    }
}

impl ToString for FurTodo {
    fn to_string(&self) -> String {
        let mut todo_string: String = self.name.to_string();

        if !self.project.is_empty() {
            todo_string += &format!(" @{}", self.project);
        }
        if !self.tags.is_empty() {
            todo_string += &format!(" #{}", self.tags);
        }
        if self.rate != 0.0 {
            todo_string += &format!(" ${:.2}", self.rate);
        }

        todo_string
    }
}

// #[derive(Clone, Debug)]
// pub struct TodoToAdd {
//     pub name: String,
//     pub project: String,
//     pub tags: String,
//     pub rate: String,
//     pub date: DateTime<Local>,
//     pub displayed_date: Date,
//     pub show_date_picker: bool,
//     pub invalid_input_error_message: String,
// }

// impl TodoToAdd {
//     pub fn new() -> Self {
//         let now = Local::now();
//         TodoToAdd {
//             name: String::new(),
//             project: String::new(),
//             tags: String::new(),
//             rate: format!("{:.2}", 0.0),
//             date: now,
//             displayed_date: Date::from(now.date_naive()),
//             show_date_picker: false,
//             invalid_input_error_message: String::new(),
//         }
//     }

//     pub fn input_error(&mut self, message: String) {
//         self.invalid_input_error_message = message;
//     }
// }

// pub struct TodoToEdit {
//     pub name: String,
//     pub new_name: String,
//     pub date: DateTime<Local>,
//     pub new_date: DateTime<Local>,
//     pub displayed_date: Date,
//     pub show_date_picker: bool,
//     pub project: String,
//     pub new_project: String,
//     pub tags: String,
//     pub new_tags: String,
//     pub rate: f32,
//     pub new_rate: String,
//     pub uid: String,
//     pub is_completed: bool,
//     pub invalid_input_error_message: String,
// }

// impl TodoToEdit {
//     pub fn new_from(todo: &FurTodo) -> Self {
//         TodoToEdit {
//             name: todo.name.clone(),
//             new_name: todo.name.clone(),
//             date: todo.date,
//             new_date: todo.date,
//             displayed_date: Date::from(todo.date.date_naive()),
//             show_date_picker: false,
//             project: todo.project.clone(),
//             new_project: todo.project.clone(),
//             tags: todo.tags.clone(),
//             new_tags: if todo.tags.is_empty() {
//                 todo.tags.clone()
//             } else {
//                 format!("#{}", todo.tags)
//             },
//             rate: todo.rate,
//             new_rate: format!("{:.2}", todo.rate),
//             uid: todo.uid.clone(),
//             is_completed: todo.is_completed,
//             invalid_input_error_message: String::new(),
//         }
//     }

//     pub fn is_changed(&self) -> bool {
//         if self.name != self.new_name.trim()
//             || self.date != self.new_date
//             || self.tags
//                 != self
//                     .new_tags
//                     .trim()
//                     .strip_prefix('#')
//                     .unwrap_or(&self.tags)
//                     .trim()
//             || self.project != self.new_project.trim()
//             || self.rate != self.new_rate.trim().parse::<f32>().unwrap_or(0.0)
//         {
//             true
//         } else {
//             false
//         }
//     }

//     pub fn input_error(&mut self, message: String) {
//         self.invalid_input_error_message = message;
//     }
// }

pub fn generate_todo_uid(name: &str, date: &DateTime<Local>) -> String {
    let input = format!("{}{}", name, date.timestamp());
    blake3::hash(input.as_bytes()).to_hex().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTodo {
    pub encrypted_data: String,
    pub nonce: String,
    pub uid: String,
    pub last_updated: i64,
}
