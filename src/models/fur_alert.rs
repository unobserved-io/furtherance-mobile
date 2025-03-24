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

#[derive(Clone, Debug)]
pub struct FurAlert {
    pub is_shown: bool,
    pub title: String,
    pub message: String,
    pub confirm_button: (String, fn()),
    pub cancel_button: Option<(String, fn())>,
}

impl FurAlert {
    pub fn new() -> Self {
        FurAlert {
            is_shown: false,
            title: "Title".to_string(),
            message: "Message".to_string(),
            confirm_button: ("Ok".to_string(), || {}),
            cancel_button: None,
        }
    }
}
