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

use crate::{database, models::fur_shortcut::FurShortcut, state};

pub fn update_all_shortcuts() {
    *state::SHORTCUTS.write() = get_all_shortcuts();
}

pub fn get_all_shortcuts() -> Vec<FurShortcut> {
    match database::shortcuts::retrieve_existing_shortcuts() {
        Ok(shortcuts) => shortcuts,
        Err(e) => {
            eprintln!("Error reading shortcuts from database: {}", e);
            vec![]
        }
    }
}
