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

use std::{sync::Arc, time::Duration};

use dioxus::{
    hooks::use_context,
    prelude::spawn,
    signals::{Readable, Writable},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::{
    constants::OFFICIAL_SERVER,
    database::{
        self,
        sync::retrieve_credentials,
        tasks::{SortBy, SortOrder},
    },
    helpers::{self, server::logout, views::settings::ServerChoices},
    models::{
        fur_shortcut::{EncryptedShortcut, FurShortcut},
        fur_task::{EncryptedTask, FurTask},
        fur_todo::{EncryptedTodo, FurTodo},
        fur_user::{FurUser, FurUserFields},
    },
    state,
};

use super::{
    encryption,
    login::{self, ApiError},
};

#[derive(Serialize, Deserialize)]
pub struct SyncRequest {
    last_sync: i64,
    device_id: String,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
    todos: Vec<EncryptedTodo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncResponse {
    pub server_timestamp: i64,
    pub tasks: Vec<EncryptedTask>,
    pub shortcuts: Vec<EncryptedShortcut>,
    pub todos: Vec<EncryptedTodo>,
    pub orphaned_tasks: Vec<String>,
    pub orphaned_shortcuts: Vec<String>,
    pub orphaned_todos: Vec<String>,
}

pub fn get_user() -> Option<FurUser> {
    match retrieve_credentials() {
        Ok(optional_user) => optional_user,
        Err(e) => {
            eprintln!("Error retrieving user credentials from database: {}", e);
            None
        }
    }
}

pub fn get_user_fields() -> FurUserFields {
    match get_user() {
        Some(user) => FurUserFields {
            email: user.email.clone(),
            encryption_key: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
            server: user.server.clone(),
            server_selection: if user.server == OFFICIAL_SERVER.to_string() {
                ServerChoices::Official
            } else {
                ServerChoices::Custom
            },
        },
        None => FurUserFields::default(),
    }
}

pub fn request_sync() {
    let state = use_context::<state::FurState>();
    let settings = state.settings.read().clone();
    // let last_sync = state.settings.read().last_sync.clone();

    let user = match state.user.read().clone() {
        Some(user) => user,
        None => {
            eprintln!("Please log in first");
            return;
        }
    };

    // TODO: Create message to show in settings or as a toast
    // self.login_message = Ok(self.localization.get_message("syncing", None));

    let encryption_key =
        match encryption::decrypt_encryption_key(&user.encrypted_key, &user.key_nonce) {
            Ok(key) => key,
            Err(e) => {
                eprintln!("Failed to decrypt encryption key (SyncWithServer): {:?}", e);
                // return messages::set_negative_temp_notice(
                //     &mut self.login_message,
                //     self.localization.get_message("error-decrypting-key", None),
                // );
                return;
            }
        };

    let needs_full_sync = settings.needs_full_sync;

    spawn(
        async move {
            let new_tasks: Vec<FurTask>;
            let new_shortcuts: Vec<FurShortcut>;
            let new_todos: Vec<FurTodo>;

            if needs_full_sync {
                new_tasks =
                    database::tasks::retrieve_all_tasks(SortBy::StartTime, SortOrder::Ascending)
                        .unwrap_or_default();
                new_shortcuts = database::shortcuts::retrieve_all_shortcuts().unwrap_or_default();
                new_todos = database::todos::retrieve_all_todos().unwrap_or_default();
            } else {
                new_tasks = database::tasks::retrieve_tasks_since_timestamp(settings.last_sync)
                    .unwrap_or_default();
                new_shortcuts =
                    database::shortcuts::retrieve_shortcuts_since_timestamp(settings.last_sync)
                        .unwrap_or_default();
                new_todos = database::todos::retrieve_todos_since_timestamp(settings.last_sync)
                    .unwrap_or_default();
            }

            let encrypted_tasks: Vec<EncryptedTask> = new_tasks
                .into_iter()
                .filter_map(|task| match encryption::encrypt(&task, &encryption_key) {
                    Ok((encrypted_data, nonce)) => Some(EncryptedTask {
                        encrypted_data,
                        nonce,
                        uid: task.uid,
                        last_updated: task.last_updated,
                    }),
                    Err(e) => {
                        eprintln!("Failed to encrypt task: {:?}", e);
                        None
                    }
                })
                .collect();

            let encrypted_shortcuts: Vec<EncryptedShortcut> = new_shortcuts
                .into_iter()
                .filter_map(
                    |shortcut| match encryption::encrypt(&shortcut, &encryption_key) {
                        Ok((encrypted_data, nonce)) => Some(EncryptedShortcut {
                            encrypted_data,
                            nonce,
                            uid: shortcut.uid,
                            last_updated: shortcut.last_updated,
                        }),
                        Err(e) => {
                            eprintln!("Failed to encrypt shortcut: {:?}", e);
                            None
                        }
                    },
                )
                .collect();

            let encrypted_todos: Vec<EncryptedTodo> = new_todos
                .into_iter()
                .filter_map(|todo| match encryption::encrypt(&todo, &encryption_key) {
                    Ok((encrypted_data, nonce)) => Some(EncryptedTodo {
                        encrypted_data,
                        nonce,
                        uid: todo.uid,
                        last_updated: todo.last_updated,
                    }),
                    Err(e) => {
                        eprintln!("Failed to encrypt todo: {:?}", e);
                        None
                    }
                })
                .collect();

            let sync_count =
                encrypted_tasks.len() + encrypted_shortcuts.len() + encrypted_todos.len();

            let sync_result = sync_with_server(
                &user,
                settings.last_sync,
                encrypted_tasks,
                encrypted_shortcuts,
                encrypted_todos,
            )
            .await;

            process_sync_result((sync_result, sync_count));
            // (sync_result, sync_count)
        }, // TODO: Message::SyncComplete,
    );
}

pub async fn sync_with_server(
    user: &FurUser,
    last_sync: i64,
    tasks: Vec<EncryptedTask>,
    shortcuts: Vec<EncryptedShortcut>,
    todos: Vec<EncryptedTodo>,
) -> Result<SyncResponse, ApiError> {
    let client = Client::new();
    let device_id = encryption::generate_device_id().map_err(|e| {
        eprintln!("Failed to create device id for logout: {:?}", e);
        ApiError::Device("Failed to generate device ID".to_string())
    })?;

    let sync_request = SyncRequest {
        last_sync,
        device_id,
        tasks,
        shortcuts,
        todos,
    };

    let mut response = client
        .post(format!("{}/api/sync", user.server))
        .header("Authorization", format!("Bearer {}", user.access_token))
        .json(&sync_request)
        .send()
        .await
        .map_err(|e| ApiError::Network(Arc::new(e)))?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        // Try token refresh
        let new_access_token =
            login::refresh_auth_token(user.refresh_token.to_string(), &user.server).await?;
        if let Err(e) = database::sync::update_access_token(&user.email, &new_access_token) {
            return Err(ApiError::TokenRefresh(e.to_string()));
        }

        // Retry with new token
        response = client
            .post(format!("{}/api/sync", user.server))
            .header("Authorization", format!("Bearer {}", new_access_token))
            .json(&sync_request)
            .send()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))?;
    }

    if response.status().is_success() {
        response
            .json()
            .await
            .map_err(|e| ApiError::Network(Arc::new(e)))
    } else {
        if let Ok(error) = response.json::<serde_json::Value>().await {
            if let Some(error_type) = error.get("error").and_then(|e| e.as_str()) {
                if error_type == "inactive_subscription" {
                    return Err(ApiError::InactiveSubscription(
                        error
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("Subscription inactive")
                            .to_string(),
                    ));
                }
            }
        }
        Err(ApiError::Server("Sync failed".into()))
    }
}

fn process_sync_result(sync_result: (Result<SyncResponse, ApiError>, usize)) {
    let mut state = use_context::<state::FurState>();
    let mut settings = state.settings.read().clone();

    match sync_result {
        (Ok(response), mut sync_count) => {
            let user = match state.user.read().clone() {
                Some(user) => user,
                None => {
                    eprintln!("Please log in first");
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization.get_message("log-in-first", None),
                    // );
                    return;
                }
            };

            let encryption_key =
                match encryption::decrypt_encryption_key(&user.encrypted_key, &user.key_nonce) {
                    Ok(key) => key,
                    Err(e) => {
                        eprintln!("Failed to decrypt encryption key (SyncComplete): {:?}", e);
                        // return messages::set_negative_temp_notice(
                        //     &mut self.login_message,
                        //     self.localization.get_message("error-decrypting-key", None),
                        // );
                        return;
                    }
                };

            // Decrypt and process server tasks
            for encrypted_task in response.tasks {
                match encryption::decrypt::<FurTask>(
                    &encrypted_task.encrypted_data,
                    &encrypted_task.nonce,
                    &encryption_key,
                ) {
                    Ok(server_task) => {
                        match database::tasks::retrieve_task_by_id(&server_task.uid) {
                            Ok(Some(client_task)) => {
                                // Task exists - update it if it changed
                                if server_task.last_updated > client_task.last_updated {
                                    if let Err(e) = database::tasks::update_task(&server_task) {
                                        eprintln!("Error updating task from server: {}", e);
                                    } else {
                                        sync_count += 1;
                                    }
                                }
                            }
                            Ok(None) => {
                                // Task does not exist - insert it
                                if let Err(e) = database::tasks::insert_task(&server_task) {
                                    eprintln!("Error writing new task from server: {}", e);
                                } else {
                                    sync_count += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error checking for existing task from server: {}", e)
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to decrypt task: {:?}", e),
                }
            }

            // Decrypt and process server shortcuts
            for encrypted_shortcut in response.shortcuts {
                match encryption::decrypt::<FurShortcut>(
                    &encrypted_shortcut.encrypted_data,
                    &encrypted_shortcut.nonce,
                    &encryption_key,
                ) {
                    Ok(server_shortcut) => {
                        match database::shortcuts::retrieve_shortcut_by_id(&server_shortcut.uid) {
                            Ok(Some(client_shortcut)) => {
                                // Shortcut exists - update it if it changed
                                if server_shortcut.last_updated > client_shortcut.last_updated {
                                    if let Err(e) =
                                        database::shortcuts::update_shortcut(&server_shortcut)
                                    {
                                        eprintln!("Error updating shortcut from server: {}", e);
                                    } else {
                                        sync_count += 1;
                                    }
                                }
                            }
                            Ok(None) => {
                                // Shortcut does not exist - insert it
                                if let Err(e) =
                                    database::shortcuts::insert_shortcut(&server_shortcut)
                                {
                                    eprintln!("Error writing new shortcut from server: {}", e);
                                } else {
                                    sync_count += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error checking for existing shortcut from server: {}", e)
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to decrypt shortcut: {:?}", e),
                }
            }

            // Decrypt and process server todos
            for encrypted_todo in response.todos {
                match encryption::decrypt::<FurTodo>(
                    &encrypted_todo.encrypted_data,
                    &encrypted_todo.nonce,
                    &encryption_key,
                ) {
                    Ok(server_todo) => {
                        match database::todos::retrieve_todo_by_id(&server_todo.uid) {
                            Ok(Some(client_todo)) => {
                                // Todo exists - update it if it changed
                                if server_todo.last_updated > client_todo.last_updated {
                                    if let Err(e) = database::todos::update_todo(&server_todo) {
                                        eprintln!("Error updating todo from server: {}", e);
                                    } else {
                                        sync_count += 1;
                                    }
                                }
                            }
                            Ok(None) => {
                                // Todo does not exist - insert it
                                if let Err(e) = database::todos::insert_todo(&server_todo) {
                                    eprintln!("Error writing new todo from server: {}", e);
                                } else {
                                    sync_count += 1;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error checking for existing todo from server: {}", e)
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to decrypt todo: {:?}", e),
                }
            }

            // Update last sync timestamp
            settings.last_sync = response.server_timestamp;
            state.settings.set(settings.clone());

            // If the database_id changed, send all tasks, or if the server has orphaned tasks, re-sync those
            if !response.orphaned_tasks.is_empty()
                || !response.orphaned_shortcuts.is_empty()
                || !response.orphaned_todos.is_empty()
            {
                let last_sync = settings.last_sync;

                let orphaned_tasks = if !response.orphaned_tasks.is_empty() {
                    database::tasks::retrieve_orphaned_tasks(response.orphaned_tasks)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                let orphaned_shortcuts = if !response.orphaned_shortcuts.is_empty() {
                    database::shortcuts::retrieve_orphaned_shortcuts(response.orphaned_shortcuts)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                let orphaned_todos = if !response.orphaned_todos.is_empty() {
                    database::todos::retrieve_orphaned_todos(response.orphaned_todos)
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };

                if !orphaned_tasks.is_empty()
                    || !orphaned_shortcuts.is_empty()
                    || !orphaned_todos.is_empty()
                {
                    spawn(async move {
                        let encrypted_tasks: Vec<EncryptedTask> = orphaned_tasks
                            .into_iter()
                            .filter_map(|task| match encryption::encrypt(&task, &encryption_key) {
                                Ok((encrypted_data, nonce)) => Some(EncryptedTask {
                                    encrypted_data,
                                    nonce,
                                    uid: task.uid,
                                    last_updated: task.last_updated,
                                }),
                                Err(e) => {
                                    eprintln!("Failed to encrypt task: {:?}", e);
                                    None
                                }
                            })
                            .collect();

                        let encrypted_shortcuts: Vec<EncryptedShortcut> = orphaned_shortcuts
                            .into_iter()
                            .filter_map(|shortcut| {
                                match encryption::encrypt(&shortcut, &encryption_key) {
                                    Ok((encrypted_data, nonce)) => Some(EncryptedShortcut {
                                        encrypted_data,
                                        nonce,
                                        uid: shortcut.uid,
                                        last_updated: shortcut.last_updated,
                                    }),
                                    Err(e) => {
                                        eprintln!("Failed to encrypt shortcut: {:?}", e);
                                        None
                                    }
                                }
                            })
                            .collect();

                        let encrypted_todos: Vec<EncryptedTodo> = orphaned_todos
                            .into_iter()
                            .filter_map(|todo| match encryption::encrypt(&todo, &encryption_key) {
                                Ok((encrypted_data, nonce)) => Some(EncryptedTodo {
                                    encrypted_data,
                                    nonce,
                                    uid: todo.uid,
                                    last_updated: todo.last_updated,
                                }),
                                Err(e) => {
                                    eprintln!("Failed to encrypt todo: {:?}", e);
                                    None
                                }
                            })
                            .collect();

                        sync_count += encrypted_tasks.len()
                            + encrypted_shortcuts.len()
                            + encrypted_todos.len();

                        let sync_result = sync_with_server(
                            &user,
                            last_sync,
                            encrypted_tasks,
                            encrypted_shortcuts,
                            encrypted_todos,
                        )
                        .await;

                        process_sync_result((sync_result, sync_count));
                    });
                    return;
                }
            }

            settings.needs_full_sync = false;
            state.settings.set(settings.clone());

            // TODO: Run these async
            spawn(async move {
                helpers::views::task_history::update_task_history(settings.days_to_show);
                helpers::views::todos::update_all_todos();
                helpers::views::shortcuts::update_all_shortcuts();
            });
            // TODO: Set notice
            // tasks.push(messages::set_positive_temp_notice(
            //     &mut self.login_message,
            //     self.localization.get_message(
            //         "sync-successful",
            //         Some(&HashMap::from([("count", FluentValue::from(sync_count))])),
            //     ),
            // ));
        }
        (Err(ApiError::TokenRefresh(msg)), _) if msg == "Failed to refresh token" => {
            eprintln!("Sync error. Credentials have changed. Log in again.");
            if let Some(user) = state.user.read().clone() {
                spawn(async move {
                    logout::server_logout(&user).await;
                    match database::sync::delete_all_credentials() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Error deleting user credentials: {}", e),
                    };
                    state.user.set(None);
                    state.user_fields.set(FurUserFields::default());
                    // TODO: Set notice
                    // return messages::set_negative_temp_notice(
                    //     &mut self.login_message,
                    //     self.localization.get_message("reauthenticate-error", None),
                    // );
                });
            }
        }
        (Err(ApiError::InactiveSubscription(msg)), _) => {
            eprintln!("Sync error: {}", msg);
            // TODO: Set notice
            // return messages::set_negative_temp_notice(
            //     &mut self.login_message,
            //     self.localization.get_message("subscription-inactive", None),
            // );
        }
        (Err(e), _) => {
            eprintln!("Sync error: {:?}", e);
            // TODO: Set notice
            // return messages::set_negative_temp_notice(
            //     &mut self.login_message,
            //     self.localization.get_message("sync-failed", None),
            // );
        }
    }
}

pub fn sync_after_change() {
    if use_context::<state::FurState>().user.read().is_some() {
        spawn(async {
            // Small delay to allow any pending DB operations to complete
            time::sleep(Duration::from_secs(1)).await;
        });
    }
}

pub fn reset_user() {
    match database::sync::delete_all_credentials() {
        Ok(_) => {}
        Err(e) => eprintln!("Error deleting user credentials: {}", e),
    };
    use_context::<state::FurState>().user.set(None);
}
