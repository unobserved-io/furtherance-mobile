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

use std::sync::Arc;

use dioxus::{
    hooks::use_context,
    prelude::spawn,
    signals::{Readable, Writable},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{database, helpers::server::sync::reset_user, models::fur_user::FurUserFields, state};

use super::encryption::{self, generate_device_id};

#[derive(Clone, Debug)]
pub enum ApiError {
    Auth(String),
    Device(String),
    InactiveSubscription(String),
    Network(Arc<reqwest::Error>),
    Server(String),
    TokenRefresh(String),
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub encryption_key: String,
    pub device_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
struct RefreshRequest {
    refresh_token: String,
    device_id: String,
}

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
}

pub fn login_button_pressed() {
    let mut state = use_context::<state::FurState>();
    let mut user_fields = state.user_fields.read().clone();
    user_fields.server = user_fields.server.clone().trim_end_matches('/').to_string();
    state.user_fields.set(user_fields);

    let user_fields = state.user_fields.read().clone();
    let email = user_fields.email.clone();
    let encryption_key = user_fields.encryption_key.clone();
    let server = user_fields.server.clone();

    // TODO: Set login message
    // self.login_message = Ok(self.localization.get_message("logging-in", None));

    spawn(async {
        let login_response = login(email, encryption_key, server).await;
        complete_login(login_response);
    });
}

pub async fn login(
    email: String,
    encryption_key: String,
    server: String,
) -> Result<LoginResponse, ApiError> {
    let client = Client::new();
    let device_id = match generate_device_id() {
        Ok(id) => id,
        Err(_) => return Err(ApiError::Device("Failed to generate device ID".into())),
    };

    let response = client
        .post(format!("{}/api/login", server))
        .json(&LoginRequest {
            email,
            encryption_key,
            device_id,
        })
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))
    } else {
        Err(ApiError::Auth("Invalid credentials".into()))
    }
}

fn complete_login(response_result: Result<LoginResponse, ApiError>) {
    let mut state = use_context::<state::FurState>();
    match response_result {
        Ok(response) => {
            // Encrypt encryption key with device-specific key
            let (encrypted_key, key_nonce) = match encryption::encrypt_encryption_key(
                &state.user_fields.read().encryption_key,
            ) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Error encrypting key: {:?}", e);
                    match database::sync::delete_all_credentials() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error deleting user credentials: {}", e),
                    };
                    state.user.set(None);
                    // TODO: Error message
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization
                    //         .get_message("error-storing-credentials", None),
                    // );
                    return;
                }
            };

            // Store credentials
            let user_fields_clone = state.user_fields.read().clone();
            if let Err(e) = database::sync::store_credentials(
                &user_fields_clone.email,
                &encrypted_key,
                &key_nonce,
                &response.access_token,
                &response.refresh_token,
                &user_fields_clone.server,
            ) {
                eprintln!("Error storing user credentials: {}", e);
                reset_user();
                // TODO: Error message
                // return messages::set_negative_temp_notice(
                //     &mut self.login_message,
                //     self.localization
                //         .get_message("error-storing-credentials", None),
                // );
                return;
            }

            // Always do a full sync after login
            let mut settings_clone = state.settings.read().clone();
            match settings_clone.change_needs_full_sync(&true) {
                Ok(_) => state.settings.set(settings_clone),
                Err(e) => eprintln!("Error changing needs_full_sync: {}", e),
            };

            let key_length = state.user_fields.read().encryption_key.len();

            // Load new user credentials from database
            match database::sync::retrieve_credentials() {
                Ok(optional_user) => state.user.set(optional_user),
                Err(e) => {
                    eprintln!("Error retrieving user credentials from database: {}", e);
                    reset_user();
                    // TODO: Error message
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization
                    //         .get_message("error-storing-credentials", None),
                    // );
                    return;
                }
            };

            if let Some(user) = state.user.read().clone() {
                let mut user_fields_clone = state.user_fields.read().clone();
                user_fields_clone.email = user.email;
                user_fields_clone.encryption_key = "x".repeat(key_length);
                user_fields_clone.server = user.server;
                state.user_fields.set(user_fields_clone);

                // TODO: Complete message
                // tasks.push(messages::set_positive_temp_notice(
                //     &mut self.login_message,
                //     self.localization.get_message("login-successful", None),
                // ));
                super::sync::sync_after_change();
            }
        }
        Err(e) => {
            eprintln!("Error logging in: {:?}", e);
            reset_user();
            match e {
                ApiError::Network(e) if e.to_string() == "builder error" => {
                    // TODO: Error message
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization
                    //         .get_message("server-must-contain-protocol", None),
                    // );
                    return;
                }
                _ => {
                    // TODO: Error message
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization.get_message("login-failed", None),
                    // );
                    return;
                }
            }
        }
    }
}

pub async fn refresh_auth_token(refresh_token: String, server: &str) -> Result<String, ApiError> {
    let client = Client::new();
    let device_id = match generate_device_id() {
        Ok(id) => id,
        Err(_) => return Err(ApiError::Device("Failed to generate device ID".into())),
    };

    let response = client
        .post(format!("{}/api/refresh", server))
        .json(&RefreshRequest {
            refresh_token,
            device_id,
        })
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status().is_success() {
        let refresh_response = response
            .json::<RefreshResponse>()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))?;
        Ok(refresh_response.access_token)
    } else {
        Err(ApiError::TokenRefresh("Failed to refresh token".into()))
    }
}
