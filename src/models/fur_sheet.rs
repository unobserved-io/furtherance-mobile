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

use super::{
    fur_shortcut::FurShortcut, fur_task::FurTask, fur_task_group::FurTaskGroup, fur_todo::FurTodo,
};

#[derive(Clone, Debug)]
pub struct FurSheet {
    pub new_task_is_shown: bool,
    pub new_shortcut_is_shown: bool,
    pub new_todo_is_shown: bool,
    pub group_details_sheet: Option<FurTaskGroup>,
    pub task_edit_sheet: Option<FurTask>,
    pub add_to_group_sheet: Option<FurTaskGroup>,
    pub edit_group_sheet: Option<FurTaskGroup>,
    pub edit_todo_sheet: Option<FurTodo>,
    pub edit_shortcut_sheet: Option<FurShortcut>,
}

impl FurSheet {
    pub fn new() -> Self {
        FurSheet {
            new_task_is_shown: false,
            new_shortcut_is_shown: false,
            new_todo_is_shown: false,
            group_details_sheet: None,
            task_edit_sheet: None,
            add_to_group_sheet: None,
            edit_group_sheet: None,
            edit_todo_sheet: None,
            edit_shortcut_sheet: None,
        }
    }
}
