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

use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

use crate::models::fur_settings;

pub fn get_directory() -> PathBuf {
    let mut path = fur_settings::get_data_path();
    path.extend(&["furtherance.db"]);
    path
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut stmt = conn.prepare(&format!(
        "SELECT COUNT(*) FROM pragma_table_info('{}') WHERE name = ?",
        table
    ))?;
    let count: i64 = stmt.query_row([column], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn db_init() -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY,
            task_name TEXT,
            start_time TIMESTAMP,
            stop_time TIMESTAMP,
            tags TEXT,
            project TEXT,
            rate REAL,
            currency TEXT,
            uid TEXT,
            is_deleted BOOLEAN DEFAULT 0,
            last_updated INTEGER DEFAULT 0
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS shortcuts (
            id INTEGER PRIMARY KEY,
            name TEXT,
            tags TEXT,
            project TEXT,
            rate REAL,
            currency TEXT,
            color_hex TEXT,
            uid TEXT,
            is_deleted BOOLEAN DEFAULT 0,
            last_updated INTEGER DEFAULT 0
        );",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
            email TEXT PRIMARY KEY,
            encrypted_key TEXT NOT NULL,
            key_nonce TEXT NOT NULL,
            access_token TEXT NOT NULL,
            refresh_token TEXT NOT NULL,
            server TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            project TEXT,
            tags TEXT,
            rate REAL,
            currency TEXT,
            date TIMESTAMP,
            uid TEXT,
            is_completed BOOLEAN DEFAULT 0,
            is_deleted BOOLEAN DEFAULT 0,
            last_updated INTEGER DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS persistence (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            is_running BOOLEAN DEFAULT 0,
            task_input TEXT,
            start_time TIMESTAMP
        )",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO persistence (
            id,
            is_running,
            task_input,
            start_time
        ) values (1, ?1, NULL, NULL)",
        params![false],
    )?;

    Ok(())
}
