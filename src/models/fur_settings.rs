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

use crate::helpers::view_enums::FurView;

use config::{Config, ConfigError, File};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FurSettings {
    pub chosen_idle_time: i64,
    pub days_to_show: i64,
    pub default_view: FurView,
    pub dynamic_total: bool,
    #[serde(default)]
    pub first_run: bool,
    pub last_sync: i64,
    pub needs_full_sync: bool,
    pub notify_of_sync: bool,
    pub notify_on_idle: bool,
    pub notify_reminder: bool,
    pub notify_reminder_interval: u16,
    pub pomodoro: bool,
    pub pomodoro_break_length: i64,
    pub pomodoro_extended_breaks: bool,
    pub pomodoro_extended_break_interval: u16,
    pub pomodoro_extended_break_length: i64,
    pub pomodoro_length: i64,
    pub pomodoro_notification_alarm_sound: bool,
    pub pomodoro_snooze_length: i64,
    pub show_chart_average_earnings: bool,
    pub show_chart_average_time: bool,
    pub show_chart_breakdown_by_selection: bool,
    pub show_chart_earnings: bool,
    pub show_chart_selection_earnings: bool,
    pub show_chart_selection_time: bool,
    pub show_chart_time_recorded: bool,
    pub show_chart_total_earnings_box: bool,
    pub show_chart_total_time_box: bool,
    pub show_daily_time_total: bool,
    pub show_delete_confirmation: bool,
    pub show_seconds: bool,
    pub show_task_earnings: bool,
    pub show_task_project: bool,
    pub show_task_tags: bool,
    pub show_todo_project: bool,
    pub show_todo_rate: bool,
    pub show_todo_tags: bool,
}

impl Default for FurSettings {
    fn default() -> Self {
        FurSettings {
            chosen_idle_time: 6,
            days_to_show: 365,
            default_view: FurView::Timer,
            dynamic_total: false,
            first_run: true,
            last_sync: 0,
            needs_full_sync: true,
            notify_of_sync: true,
            notify_on_idle: true,
            notify_reminder: false,
            notify_reminder_interval: 10,
            pomodoro: false,
            pomodoro_break_length: 5,
            pomodoro_extended_breaks: false,
            pomodoro_extended_break_interval: 4,
            pomodoro_extended_break_length: 25,
            pomodoro_length: 25,
            pomodoro_notification_alarm_sound: true,
            pomodoro_snooze_length: 5,
            show_chart_average_earnings: true,
            show_chart_average_time: true,
            show_chart_breakdown_by_selection: true,
            show_chart_earnings: true,
            show_chart_selection_earnings: true,
            show_chart_selection_time: true,
            show_chart_time_recorded: true,
            show_chart_total_earnings_box: true,
            show_chart_total_time_box: true,
            show_daily_time_total: true,
            show_delete_confirmation: true,
            show_seconds: true,
            show_task_earnings: true,
            show_task_project: true,
            show_task_tags: true,
            show_todo_project: true,
            show_todo_rate: true,
            show_todo_tags: true,
        }
    }
}

impl FurSettings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        let config_path: PathBuf = get_settings_path();
        let path_str = config_path.to_string_lossy().into_owned();

        // Check if the configuration file exists
        if config_path.exists() {
            builder = builder.add_source(File::with_name(&path_str));
        } else {
            // Create the default configuration file
            let default_settings = FurSettings::default();
            let toml =
                toml::to_string(&default_settings).expect("Failed to serialize default settings");
            fs::write(&config_path, &toml).expect("Failed to write default config file");

            builder = builder.add_source(File::from_str(&toml, config::FileFormat::Toml));
        }

        let config = builder.build()?;
        let settings: FurSettings = config.try_deserialize()?;

        // If the configuration file existed and we added a new setting, save it
        if config_path.exists() {
            if let Err(e) = settings.save() {
                eprintln!("Error saving updated settings: {e}");
            }
        }

        Ok(settings)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let toml = toml::to_string(self).expect("Failed to serialize settings");
        fs::write(get_settings_path(), toml)
    }

    pub fn change_chosen_idle_time(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.chosen_idle_time = value.to_owned();
        self.save()
    }

    pub fn change_days_to_show(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.days_to_show = value.to_owned();
        self.save()
    }

    pub fn change_default_view(&mut self, value: &FurView) -> Result<(), std::io::Error> {
        self.default_view = value.to_owned();
        self.save()
    }

    pub fn change_first_run(&mut self, value: bool) -> Result<(), std::io::Error> {
        self.first_run = value;
        self.save()
    }

    pub fn change_last_sync(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.last_sync = value.to_owned();
        self.save()
    }

    pub fn change_needs_full_sync(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.needs_full_sync = value.to_owned();
        self.save()
    }

    pub fn change_notify_of_sync(&mut self, value: bool) -> Result<(), std::io::Error> {
        self.notify_of_sync = value;
        self.save()
    }

    pub fn change_notify_on_idle(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.notify_on_idle = value.to_owned();
        self.save()
    }

    pub fn change_notify_reminder(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.notify_reminder = value.to_owned();
        self.save()
    }

    pub fn change_notify_reminder_interval(&mut self, value: &u16) -> Result<(), std::io::Error> {
        self.notify_reminder_interval = value.to_owned();
        self.save()
    }

    pub fn change_dynamic_total(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.dynamic_total = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.pomodoro = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_break_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_break_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_extended_breaks(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.pomodoro_extended_breaks = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_extended_break_interval(
        &mut self,
        value: &u16,
    ) -> Result<(), std::io::Error> {
        self.pomodoro_extended_break_interval = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_extended_break_length(
        &mut self,
        value: &i64,
    ) -> Result<(), std::io::Error> {
        self.pomodoro_extended_break_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_snooze_length(&mut self, value: &i64) -> Result<(), std::io::Error> {
        self.pomodoro_snooze_length = value.to_owned();
        self.save()
    }

    pub fn change_pomodoro_notification_alarm_sound(
        &mut self,
        value: &bool,
    ) -> Result<(), std::io::Error> {
        self.pomodoro_notification_alarm_sound = value.to_owned();
        self.save()
    }

    pub fn change_show_daily_time_total(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_daily_time_total = value.to_owned();
        self.save()
    }

    pub fn change_show_delete_confirmation(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_delete_confirmation = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_average_earnings(
        &mut self,
        value: &bool,
    ) -> Result<(), std::io::Error> {
        self.show_chart_average_earnings = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_average_time(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_chart_average_time = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_breakdown_by_selection(
        &mut self,
        value: &bool,
    ) -> Result<(), std::io::Error> {
        self.show_chart_breakdown_by_selection = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_earnings(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_chart_earnings = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_selection_earnings(
        &mut self,
        value: &bool,
    ) -> Result<(), std::io::Error> {
        self.show_chart_selection_earnings = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_selection_time(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_chart_selection_time = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_time_recorded(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_chart_time_recorded = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_total_earnings_box(
        &mut self,
        value: &bool,
    ) -> Result<(), std::io::Error> {
        self.show_chart_total_earnings_box = value.to_owned();
        self.save()
    }

    pub fn change_show_chart_total_time_box(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_chart_total_time_box = value.to_owned();
        self.save()
    }

    pub fn change_show_seconds(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_seconds = value.to_owned();
        self.save()
    }

    pub fn change_show_task_earnings(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_task_earnings = value.to_owned();
        self.save()
    }

    pub fn change_show_task_project(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_task_project = value.to_owned();
        self.save()
    }

    pub fn change_show_task_tags(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_task_tags = value.to_owned();
        self.save()
    }

    pub fn change_show_todo_project(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_todo_project = value.to_owned();
        self.save()
    }

    pub fn change_show_todo_rate(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_todo_rate = value.to_owned();
        self.save()
    }

    pub fn change_show_todo_tags(&mut self, value: &bool) -> Result<(), std::io::Error> {
        self.show_todo_tags = value.to_owned();
        self.save()
    }
}

pub fn get_data_path() -> PathBuf {
    if let Some(base_dirs) = BaseDirs::new() {
        println!("BaseDirs data_dir: {:?}", base_dirs.data_dir());
        let path = PathBuf::from(base_dirs.data_dir());
        std::fs::create_dir_all(&path).expect("Could not create data directory");
        return path;
    } else {
        let fallback = PathBuf::from(".");
        std::fs::create_dir_all(&fallback).ok();
        fallback
    }
}

fn get_settings_path() -> PathBuf {
    let mut path = get_data_path();
    path.extend(&["settings.toml"]);
    path
}
