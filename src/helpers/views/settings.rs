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

use crate::{loc, localization::Localization};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServerChoices {
    Official,
    Custom,
}

impl ServerChoices {
    pub fn all_as_strings() -> Vec<String> {
        vec![
            ServerChoices::Official.to_string(),
            ServerChoices::Custom.to_string(),
        ]
    }
}

impl std::fmt::Display for ServerChoices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ServerChoices::Official => loc!("official-server"),
                ServerChoices::Custom => loc!("custom"),
            }
        )
    }
}
