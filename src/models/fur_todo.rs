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
