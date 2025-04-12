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

use crate::models::{fur_task::FurTask, fur_task_group::FurTaskGroup};

use super::init::get_directory;

#[derive(Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Ascending
    }
}

impl SortOrder {
    fn to_sqlite(&self) -> &str {
        match self {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SortBy {
    StartTime,
    StopTime,
    TaskName,
}

impl Default for SortBy {
    fn default() -> Self {
        Self::StartTime
    }
}

impl SortBy {
    fn to_sqlite(&self) -> &str {
        match self {
            Self::StartTime => "start_time",
            Self::StopTime => "stop_time",
            Self::TaskName => "task_name",
        }
    }
}

pub fn insert_task(task: &FurTask) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "INSERT INTO tasks (
            task_name,
            start_time,
            stop_time,
            tags,
            project,
            rate,
            currency,
            uid,
            is_deleted,
            last_updated
        ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            task.name,
            task.start_time.to_rfc3339(),
            task.stop_time.to_rfc3339(),
            task.tags,
            task.project,
            task.rate,
            task.currency,
            task.uid,
            task.is_deleted,
            task.last_updated
        ],
    )?;

    Ok(())
}

pub fn insert_tasks(tasks: &[FurTask]) -> Result<()> {
    let mut conn = Connection::open(get_directory())?;

    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT INTO tasks (
                task_name,
                start_time,
                stop_time,
                tags,
                project,
                rate,
                currency,
                uid,
                is_deleted,
                last_updated
            ) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        )?;

        for task in tasks {
            stmt.execute(params![
                task.name,
                task.start_time.to_rfc3339(),
                task.stop_time.to_rfc3339(),
                task.tags,
                task.project,
                task.rate,
                task.currency,
                task.uid,
                task.is_deleted,
                task.last_updated
            ])?;
        }
    }

    tx.commit()?;

    Ok(())
}

pub fn retrieve_all_tasks(sort: SortBy, order: SortOrder) -> Result<Vec<FurTask>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM tasks ORDER BY {0} {1}",
            sort.to_sqlite(),
            order.to_sqlite()
        )
        .as_str(),
    )?;
    let mut rows = stmt.query(params![])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn retrieve_all_existing_tasks(
    sort: SortBy,
    order: SortOrder,
) -> Result<Vec<FurTask>, rusqlite::Error> {
    // Retrieve all tasks from the database
    let conn = Connection::open(get_directory())?;

    let mut stmt = conn.prepare(
        format!(
            "SELECT * FROM tasks WHERE is_deleted = 0 ORDER BY {0} {1}",
            sort.to_sqlite(),
            order.to_sqlite()
        )
        .as_str(),
    )?;
    let mut rows = stmt.query(params![])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn retrieve_tasks_by_date_range(start_date: String, end_date: String) -> Result<Vec<FurTask>> {
    let conn = Connection::open(get_directory())?;
    let mut stmt = conn.prepare(
        "SELECT * FROM tasks WHERE start_time BETWEEN ?1 AND ?2 AND is_deleted = 0 ORDER BY start_time ASC",
    )?;
    let mut rows = stmt.query(params![start_date, end_date])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

/// Retrieve a limited number of days worth of tasks
pub fn retrieve_tasks_with_day_limit(
    days: i64,
    sort: SortBy,
    order: SortOrder,
) -> Result<Vec<FurTask>> {
    let conn = Connection::open(get_directory())?;

    // Construct the query string dynamically
    let query = format!(
        "SELECT * FROM tasks WHERE start_time >= date('now', ?) AND is_deleted = 0 ORDER BY {} {}",
        sort.to_sqlite(),
        order.to_sqlite()
    );

    let mut stmt = conn.prepare(&query)?;
    let mut rows = stmt.query(params![format!("-{} days", days - 1)])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn retrieve_task_by_id(uid: &String) -> Result<Option<FurTask>> {
    let conn = Connection::open(get_directory())?;
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE uid = ?")?;
    let mut rows = stmt.query_map([uid.to_string()], |row| {
        Ok(FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        })
    })?;

    match rows.next() {
        Some(Ok(task)) => Ok(Some(task)),
        Some(Err(e)) => Err(e.into()),
        None => Ok(None),
    }
}

pub fn retrieve_tasks_since_timestamp(timestamp: i64) -> Result<Vec<FurTask>, rusqlite::Error> {
    let conn = Connection::open(get_directory())?;

    let mut stmt =
        conn.prepare("SELECT * FROM tasks WHERE last_updated >= ? ORDER BY last_updated ASC")?;
    let mut rows = stmt.query(params![timestamp])?;

    let mut tasks_vec: Vec<FurTask> = Vec::new();

    while let Some(row) = rows.next()? {
        let fur_task = FurTask {
            name: row.get(1)?,
            start_time: row.get(2)?,
            stop_time: row.get(3)?,
            tags: row.get(4)?,
            project: row.get(5)?,
            rate: row.get(6)?,
            currency: row.get(7).unwrap_or(String::new()),
            uid: row.get(8)?,
            is_deleted: row.get(9)?,
            last_updated: row.get(10)?,
        };
        tasks_vec.push(fur_task);
    }

    Ok(tasks_vec)
}

pub fn retrieve_orphaned_tasks(task_uids: Vec<String>) -> Result<Vec<FurTask>> {
    let mut conn = Connection::open(get_directory())?;
    let mut tasks = Vec::new();

    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare("SELECT * FROM tasks WHERE uid = ?")?;

        for uid in task_uids {
            let task_iter = stmt.query_map(params![uid], |row| {
                Ok(FurTask {
                    name: row.get(1)?,
                    start_time: row.get(2)?,
                    stop_time: row.get(3)?,
                    tags: row.get(4)?,
                    project: row.get(5)?,
                    rate: row.get(6)?,
                    currency: row.get(7).unwrap_or(String::new()),
                    uid: row.get(8)?,
                    is_deleted: row.get(9)?,
                    last_updated: row.get(10)?,
                })
            })?;

            // Collect any matching tasks
            for task in task_iter {
                tasks.push(task?);
            }
        }
    }

    tx.commit()?;
    Ok(tasks)
}

pub fn update_task(task: &FurTask) -> Result<()> {
    let conn = Connection::open(get_directory())?;

    conn.execute(
        "UPDATE tasks SET
            task_name = ?1,
            start_time = ?2,
            stop_time = ?3,
            tags = ?4,
            project = ?5,
            rate = ?6,
            currency = ?7,
            is_deleted = ?8,
            last_updated = ?9
        WHERE uid = ?10",
        params![
            task.name,
            task.start_time.to_rfc3339(),
            task.stop_time.to_rfc3339(),
            task.tags,
            task.project,
            task.rate,
            task.currency,
            task.is_deleted,
            task.last_updated,
            task.uid,
        ],
    )?;

    Ok(())
}

pub fn task_exists(task: &FurTask) -> Result<bool> {
    let conn = Connection::open(get_directory())?;

    let query = "
        SELECT 1 FROM tasks
        WHERE task_name = ?1
        AND start_time = ?2
        AND stop_time = ?3
        AND tags = ?4
        AND project = ?5
        AND rate = ?6
        AND currency = ?7
        AND is_deleted = ?8
        LIMIT 1
    ";

    let mut stmt = conn.prepare(query)?;

    let exists = stmt.exists(params![
        task.name,
        task.start_time.to_rfc3339(),
        task.stop_time.to_rfc3339(),
        task.tags,
        task.project,
        task.rate,
        task.currency,
        task.is_deleted,
    ])?;

    Ok(exists)
}

pub fn delete_tasks_by_ids(id_list: &[String]) -> Result<()> {
    let conn = Connection::open(get_directory())?;
    let now = chrono::Utc::now().timestamp();

    for id in id_list {
        conn.execute(
            "UPDATE tasks SET is_deleted = 1, last_updated = ?1 WHERE uid = ?2",
            params![now, id],
        )?;
    }

    Ok(())
}

pub fn update_group_of_tasks(group: &FurTaskGroup) -> Result<()> {
    let mut conn = Connection::open(get_directory())?;
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "UPDATE tasks SET
            task_name = ?1,
            tags = ?2,
            project = ?3,
            rate = ?4,
            last_updated = ?5
        WHERE uid = ?6",
        )?;

        for uid in group.all_task_ids().iter() {
            stmt.execute(params![
                group.name,
                group.tags,
                group.project,
                group.rate,
                chrono::Utc::now().timestamp(),
                uid,
            ])?;
        }
    }

    // Commit the transaction
    tx.commit()?;

    Ok(())
}
