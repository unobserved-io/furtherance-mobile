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

use crate::models::fur_todo::FurTodo;

use super::init::get_directory;

pub fn retrieve_all_todos() -> Result<Vec<FurTodo>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare("SELECT * FROM todos ORDER BY name")?;
    let mut rows = stmt.query(params![])?;

    let mut todos: Vec<FurTodo> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_todo = FurTodo {
            name: row.get(1)?,
            project: row.get(2)?,
            tags: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5).unwrap_or(String::new()),
            date: row.get(6)?,
            uid: row.get(7)?,
            is_completed: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        todos.push(fur_todo);
    }

    Ok(todos)
}

pub fn retrieve_todos_between_dates(start_date: String, end_date: String) -> Result<Vec<FurTodo>> {
    let conn = Connection::open(get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM todos WHERE date BETWEEN ?1 AND ?2 AND is_deleted = 0")?;

    let mut rows = stmt.query(params![start_date, end_date])?;

    let mut todo_vec: Vec<FurTodo> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_todo = FurTodo {
            name: row.get(1)?,
            project: row.get(2)?,
            tags: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5).unwrap_or(String::new()),
            date: row.get(6)?,
            uid: row.get(7)?,
            is_completed: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        todo_vec.push(fur_todo);
    }

    Ok(todo_vec)
}

pub fn retrieve_todos_since_timestamp(timestamp: i64) -> Result<Vec<FurTodo>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM todos WHERE last_updated >= ? ORDER BY last_updated ASC")?;
    let mut rows = stmt.query(params![timestamp])?;

    let mut todos: Vec<FurTodo> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_todo = FurTodo {
            name: row.get(1)?,
            project: row.get(2)?,
            tags: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5).unwrap_or(String::new()),
            date: row.get(6)?,
            uid: row.get(7)?,
            is_completed: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        todos.push(fur_todo);
    }

    Ok(todos)
}

pub fn retrieve_todo_by_id(uid: &String) -> Result<Option<FurTodo>> {
    let conn = Connection::open(get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM todos WHERE uid = ?")?;
    let mut rows = stmt.query_map([uid.to_string()], |row| {
        Ok(FurTodo {
            name: row.get(1)?,
            project: row.get(2)?,
            tags: row.get(3)?,
            rate: row.get(4)?,
            currency: row.get(5).unwrap_or(String::new()),
            date: row.get(6)?,
            uid: row.get(7)?,
            is_completed: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        })
    })?;

    match rows.next() {
        Some(Ok(todo)) => Ok(Some(todo)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn retrieve_orphaned_todos(todo_uids: Vec<String>) -> Result<Vec<FurTodo>> {
    let mut conn = Connection::open(get_directory())?;
    let mut todos = Vec::new();

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("SELECT * FROM todos WHERE uid = ?")?;

        for uid in todo_uids {
            let todo_iter = stmt.query_map(params![uid], |row| {
                Ok(FurTodo {
                    name: row.get(1)?,
                    project: row.get(2)?,
                    tags: row.get(3)?,
                    rate: row.get(4)?,
                    currency: row.get(5).unwrap_or(String::new()),
                    date: row.get(6)?,
                    uid: row.get(7)?,
                    is_completed: row.get(8)?,
                    is_deleted: row.get(9)?,
                    last_updated: row.get(10)?,
                })
            })?;

            // Collect any matching tasks
            for todo in todo_iter {
                todos.push(todo?);
            }
        }
    }

    tx.commit()?;
    Ok(todos)
}

pub fn update_todo(todo: &FurTodo) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE todos SET
            name = ?1,
            project = ?2,
            tags = ?3,
            rate = ?4,
            currency = ?5,
            date = ?6,
            is_completed = ?7,
            is_deleted = ?8,
            last_updated = ?9
        WHERE uid = ?10",
        params![
            todo.name,
            todo.project,
            todo.tags,
            todo.rate,
            todo.currency,
            todo.date.to_rfc3339(),
            todo.is_completed,
            todo.is_deleted,
            todo.last_updated,
            todo.uid,
        ],
    )?;

    Ok(())
}

pub fn insert_todo(todo: &FurTodo) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT INTO todos (
            name,
            project,
            tags,
            rate,
            currency,
            date,
            uid,
            is_completed,
            is_deleted,
            last_updated
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            todo.name,
            todo.project,
            todo.tags,
            todo.rate,
            todo.currency,
            todo.date.to_rfc3339(),
            todo.uid,
            todo.is_completed,
            todo.is_deleted,
            todo.last_updated
        ],
    )?;

    Ok(())
}

pub fn toggle_todo_completed(uid: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "UPDATE todos SET
            is_completed = NOT is_completed,
            last_updated = ?1
        WHERE uid = ?2",
        params![now, uid],
    )?;

    Ok(())
}

pub fn set_todo_completed(uid: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "UPDATE todos SET
            is_completed = true,
            last_updated = ?1
        WHERE uid = ?2",
        params![now, uid],
    )?;

    Ok(())
}

pub fn delete_todo_by_id(uid: &str) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    let now = chrono::Utc::now().timestamp();

    conn.execute(
        "UPDATE todos SET
            is_deleted = 1,
            last_updated = ?1
        WHERE uid = ?2",
        params![now, uid],
    )?;

    Ok(())
}
