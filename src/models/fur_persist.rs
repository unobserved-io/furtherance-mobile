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

use chrono::{DateTime, Local};

use crate::database;

#[derive(Clone, Debug, PartialEq)]
pub struct FurPersist {
    pub is_running: bool,
    pub task_input: String,
    pub start_time: DateTime<Local>,
}

pub fn reset_persisting_timer() {
    if let Err(e) = database::persistence::update_persisting_timer(&FurPersist {
        is_running: false,
        task_input: String::new(),
        start_time: Local::now(),
    }) {
        eprintln!("Error updating persisting timer: {}", e);
    }
}
