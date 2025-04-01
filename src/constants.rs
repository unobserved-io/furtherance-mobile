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

use std::sync::Mutex;

use dioxus::prelude::{asset, manganis, Asset};
use once_cell::sync::{Lazy, OnceCell};

use crate::{localization::Localization, models::fur_settings::FurSettings};

pub const ALLOWED_DB_EXTENSIONS: &[&str] =
    &["db", "sqlite", "sqlite3", "db3", "database", "data", "s3db"];
pub const DEBUG_MODE: bool = cfg!(debug_assertions);
pub const FURTHERANCE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Settings
pub const SETTINGS_MESSAGE_DURATION: u64 = 8;

// Sync
pub const OFFICIAL_SERVER: &str = "https://sync.furtherance.app";

// Assets
pub const FAVICON: Asset = asset!("/assets/favicon.ico");
pub const MAIN_CSS: Asset = asset!("/assets/css/main.css");
pub const TIMER_CSS: Asset = asset!("/assets/css/timer.css");
pub const TODO_CSS: Asset = asset!("/assets/css/todo.css");
pub const SHORTCUTS_CSS: Asset = asset!("/assets/css/shortcuts.css");
pub const SETTINGS_CSS: Asset = asset!("/assets/css/settings.css");
pub const ALERT_CSS: Asset = asset!("/assets/css/alert.css");
pub const SHEET_CSS: Asset = asset!("/assets/css/sheet.css");

// Localization
pub static LOCALIZATION: OnceCell<Localization> = OnceCell::new();
