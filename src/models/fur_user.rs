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

use crate::{constants::OFFICIAL_SERVER, helpers::views::settings::ServerChoices};

#[derive(Clone, Debug, PartialEq)]
pub struct FurUser {
    pub email: String,
    pub encrypted_key: String,
    pub key_nonce: String,
    pub access_token: String,
    pub refresh_token: String,
    pub server: String,
}

#[derive(Debug, Clone)]
pub struct FurUserFields {
    pub email: String,
    pub encryption_key: String,
    pub server: String,
    pub server_selection: ServerChoices,
}

impl Default for FurUserFields {
    fn default() -> Self {
        FurUserFields {
            email: String::new(),
            encryption_key: String::new(),
            server: OFFICIAL_SERVER.to_string(),
            server_selection: ServerChoices::Official,
        }
    }
}
