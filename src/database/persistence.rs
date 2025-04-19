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

use crate::models::fur_persist::FurPersist;

use super::init::get_directory;

pub fn retrieve_persisting_timer() -> Result<FurPersist, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM persistence")?;
    let mut rows = stmt.query(params![])?;

    if let Some(row) = rows.next()? {
        return Ok(FurPersist {
            is_running: row.get(1)?,
            task_input: row.get(2)?,
            start_time: row.get(3)?,
        });
    }

    Err(rusqlite::Error::QueryReturnedNoRows)
}

pub fn update_persisting_timer(persisting_timer: &FurPersist) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE persistence SET
            is_running = ?1,
            task_input = ?2,
            start_time = ?3
        WHERE id = 1",
        params![
            persisting_timer.is_running,
            persisting_timer.task_input,
            persisting_timer.start_time,
        ],
    )?;

    Ok(())
}

pub fn update_persisting_timer_task_input(task_input: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE persistence SET
            task_input = ?1
        WHERE id = 1",
        params![task_input,],
    )?;

    Ok(())
}
