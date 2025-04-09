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

use chrono::{Datelike, Local, NaiveDate, TimeDelta};
use dioxus::{hooks::use_context, signals::Readable};
use itertools::Itertools;
use regex::Regex;

use crate::{loc, localization::Localization, state};

pub fn seconds_to_formatted_duration(total_seconds: i64) -> String {
    if use_context::<state::FurState>()
        .settings
        .cloned()
        .show_seconds
    {
        seconds_to_hms(total_seconds)
    } else {
        seconds_to_hm(total_seconds)
    }
}

pub fn format_title_date(date: &NaiveDate) -> String {
    let today = Local::now().date_naive();
    let yesterday = today - TimeDelta::days(1);
    let tomorrow = today + TimeDelta::days(1);
    let current_year = today.year();

    if date == &today {
        loc!("today")
    } else if date == &yesterday {
        loc!("yesterday")
    } else if date == &tomorrow {
        loc!("tomorrow")
    } else if date.year() == current_year {
        date.format("%b %d").to_string()
    } else {
        date.format("%b %d, %Y").to_string()
    }
}

pub fn split_task_input(input: &str) -> (String, String, String, f32) {
    let re_name = Regex::new(r"^[^@#$]+").unwrap();
    let re_project = Regex::new(r"@([^#\$]+)").unwrap();
    let re_tags = Regex::new(r"#([^@#$]+)").unwrap();
    let re_rate = Regex::new(r"\$([^@#$]+)").unwrap();

    let name = re_name
        .find(input)
        .map_or("", |m| m.as_str().trim())
        .to_string();

    let project = re_project
        .captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or(String::new());

    let tags = re_tags
        .captures_iter(input)
        .map(|cap| cap.get(1).unwrap().as_str().trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .sorted()
        .unique()
        .collect::<Vec<String>>()
        .join(" #");

    let rate_string = re_rate
        .captures(input)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or("0.0".to_string());
    let rate: f32 = rate_string.parse().unwrap_or(0.0);

    (name, project, tags, rate)
}

fn seconds_to_hms(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    let s = total_seconds % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}

fn seconds_to_hm(total_seconds: i64) -> String {
    let h = total_seconds / 3600;
    let m = total_seconds % 3600 / 60;
    format!("{}:{:02}", h, m)
}
