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

use crate::models::fur_shortcut::FurShortcut;

use super::init::get_directory;

/// Insert a shortcut to the database
pub fn insert_shortcut(shortcut: &FurShortcut) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    conn.execute(
        "INSERT INTO shortcuts (
            name,
            tags,
            project,
            rate,
            currency,
            color_hex,
            uid,
            is_deleted,
            last_updated
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            shortcut.name,
            shortcut.tags,
            shortcut.project,
            shortcut.rate,
            shortcut.currency,
            shortcut.color_hex,
            shortcut.uid,
            shortcut.is_deleted,
            shortcut.last_updated,
        ],
    )?;

    Ok(())
}

/// Retrieve all shortcuts from the database
pub fn retrieve_all_shortcuts() -> Result<Vec<FurShortcut>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM shortcuts ORDER BY name")?;
    let mut rows = stmt.query(params![])?;

    let mut shortcuts: Vec<FurShortcut> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_shortcut = FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        };
        shortcuts.push(fur_shortcut);
    }

    Ok(shortcuts)
}

/// Retrieve all existing (not deleted) shortcuts from the database
pub fn retrieve_existing_shortcuts() -> Result<Vec<FurShortcut>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM shortcuts WHERE is_deleted = 0 ORDER BY name")?;
    let mut rows = stmt.query(params![])?;

    let mut shortcuts: Vec<FurShortcut> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_shortcut = FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        };
        shortcuts.push(fur_shortcut);
    }

    Ok(shortcuts)
}

pub fn retrieve_shortcuts_since_timestamp(
    timestamp: i64,
) -> Result<Vec<FurShortcut>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM shortcuts WHERE last_updated >= ? ORDER BY last_updated ASC")?;
    let mut rows = stmt.query(params![timestamp])?;

    let mut shortcuts_vec: Vec<FurShortcut> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_shortcut = FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        };
        shortcuts_vec.push(fur_shortcut);
    }

    Ok(shortcuts_vec)
}

pub fn retrieve_orphaned_shortcuts(shortcut_uids: Vec<String>) -> Result<Vec<FurShortcut>> {
    let mut conn = Connection::open(get_directory())?;
    let mut shortcuts = Vec::new();

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("SELECT * FROM shortcuts WHERE uid = ?")?;

        for uid in shortcut_uids {
            let shortcut_iter = stmt.query_map(params![uid], |row| {
                Ok(FurShortcut {
                    name: row.get(1)?,
                    tags: row.get(2)?,
                    project: row.get(3)?,
                    rate: row.get(4)?,
                    currency: row.get(5)?,
                    color_hex: row.get(6)?,
                    uid: row.get(7)?,
                    is_deleted: row.get(8)?,
                    last_updated: row.get(9)?,
                })
            })?;

            // Collect any matching tasks
            for shortcut in shortcut_iter {
                shortcuts.push(shortcut?);
            }
        }
    }

    tx.commit()?;
    Ok(shortcuts)
}

pub fn update_shortcut(shortcut: &FurShortcut) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE shortcuts SET
            name = (?1),
            tags = (?2),
            project = (?3),
            rate = (?4),
            currency = (?5),
            color_hex = (?6),
            is_deleted = (?7),
            last_updated = (?8)
        WHERE uid = (?9)",
        params![
            shortcut.name,
            shortcut.tags,
            shortcut.project,
            shortcut.rate,
            shortcut.currency,
            shortcut.color_hex,
            shortcut.is_deleted,
            shortcut.last_updated,
            shortcut.uid,
        ],
    )?;

    Ok(())
}

pub fn shortcut_exists(shortcut: &FurShortcut) -> Result<bool> {
    let conn = Connection::open(get_directory())?;

    let query = "
        SELECT 1 FROM shortcuts
        WHERE name = ?1
        AND tags = ?2
        AND project = ?3
        AND rate = ?4
        AND currency = ?5
        AND is_deleted = 0
        LIMIT 1
    ";

    let mut stmt = conn.prepare(query)?;

    let exists = stmt.exists(params![
        shortcut.name,
        shortcut.tags,
        shortcut.project,
        shortcut.rate,
        shortcut.currency,
    ])?;

    Ok(exists)
}

pub fn retrieve_shortcut_by_id(uid: &String) -> Result<Option<FurShortcut>> {
    let conn = Connection::open(get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM shortcuts WHERE uid = ?")?;
    let mut rows = stmt.query_map([uid.to_string()], |row| {
        Ok(FurShortcut {
            name: row.get(1)?,
            tags: row.get(2)?,
            project: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5)?,
            color_hex: row.get(6)?,
            uid: row.get(7)?,
            is_deleted: row.get(8)?,
            last_updated: row.get(9)?,
        })
    })?;

    match rows.next() {
        Some(Ok(shortcut)) => Ok(Some(shortcut)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn delete_shortcut_by_id(uid: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "UPDATE shortcuts SET is_deleted = 1, last_updated = ?1 WHERE uid = ?2",
        params![now, uid],
    )?;

    Ok(())
}
