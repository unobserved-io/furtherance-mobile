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

use crate::models::fur_user::FurUser;

use super::init::get_directory;

pub fn retrieve_credentials() -> Result<Option<FurUser>> {
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM user LIMIT 1")?;

    let result = stmt.query_row([], |row| {
        Ok(FurUser {
            email: row.get(0)?,
            encrypted_key: row.get(1)?,
            key_nonce: row.get(2)?,
            access_token: row.get(3)?,
            refresh_token: row.get(4)?,
            server: row.get(5)?,
        })
    });

    match result {
        Ok(user) => Ok(Some(user)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn store_credentials(
    email: &str,
    encrypted_key: &str,
    key_nonce: &str,
    access_token: &str,
    refresh_token: &str,
    server: &str,
) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT OR REPLACE INTO user
        (email, encrypted_key, key_nonce, access_token, refresh_token, server)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            email,
            encrypted_key,
            key_nonce,
            access_token,
            refresh_token,
            server
        ],
    )?;

    Ok(())
}

pub fn update_access_token(email: &str, new_token: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE user
         SET access_token = ?1
         WHERE email = ?2",
        params![new_token, email],
    )?;

    Ok(())
}

pub fn delete_all_credentials() -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute("DELETE FROM user", [])?;

    Ok(())
}
