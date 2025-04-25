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

use dioxus::{prelude::spawn, signals::Readable};
use reqwest::Client;
use serde::Serialize;

use crate::{
    helpers::server::encryption,
    models::fur_user::{FurUser, FurUserFields},
    state,
};

use super::sync::reset_user;

#[derive(Serialize)]
pub struct LogoutRequest {
    device_id: String,
}

pub fn logout_button_pressed() {
    let user_clone = state::USER.cloned();
    if let Some(user) = user_clone {
        spawn(async move {
            server_logout(&user).await;
            logout_complete();
        });
    }
}

pub async fn server_logout(user: &FurUser) {
    let client = Client::new();
    let device_id = match encryption::generate_device_id() {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Failed to create device id for logout: {:?}", e);
            return;
        }
    };

    if let Err(e) = client
        .post(format!("{}/api/logout", user.server))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&LogoutRequest { device_id })
        .send()
        .await
    {
        eprintln!("Failed to send logout request: {}", e);
    }
}

pub fn logout_complete() {
    reset_user();
    *state::USER_FIELDS.write() = FurUserFields::default();
    // TODO: Set message
    // return messages::set_positive_temp_notice(
    //     &mut self.login_message,
    //     self.localization.get_message("logged-out", None),
    // );
}
