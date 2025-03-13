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

use dioxus::signals::Readable;

use crate::constants::{TASK_INPUT, TIMER_IS_RUNNING};

pub fn validate_task_input(input: String) {
    // If timer is running, task can never be empty
    if TIMER_IS_RUNNING.cloned() {
        if input.trim().is_empty() {
            return;
        }
    }
    let input_trimmed = input.trim_start();
    // Doesn't start with @
    if input_trimmed.chars().next() != Some('@')
    // Doesn't start with #
    && input_trimmed.chars().next() != Some('#')
    // Doesn't start with $
    && input_trimmed.chars().next() != Some('$')
    // No more than 1 @
    && input_trimmed.chars().filter(|&c| c == '@').count() < 2
    // No more than 1 $
    && input_trimmed.chars().filter(|&c| c == '$').count() < 2
    {
        // Check if there is a $ and the subsequent part is a parseable f32
        if let Some(dollar_index) = input_trimmed.find('$') {
            let after_dollar = &input_trimmed[dollar_index + 1..];
            if after_dollar.is_empty() {
                // Allow typing the $ in the first place
                *TASK_INPUT.write() = input_trimmed.to_string();
            } else {
                // Find the parseable number right after the $
                let end_index = after_dollar.find(' ').unwrap_or(after_dollar.len());
                let number_str = &after_dollar[..end_index];
                let parsed_num = number_str.parse::<f32>();

                if parsed_num.is_ok()
                    && has_max_two_decimals(&number_str)
                    && parsed_num.unwrap_or(f32::MAX) < f32::MAX
                {
                    let remaining_str = &after_dollar[end_index..].trim_start();
                    if remaining_str.is_empty() {
                        // Allow a number to be typed after the $
                        *TASK_INPUT.write() = input_trimmed.to_string();
                    } else {
                        // Only allow a space, @, or # to be typed after the $ amount
                        if remaining_str.starts_with('@') || remaining_str.starts_with('#') {
                            *TASK_INPUT.write() = input_trimmed.to_string();
                        }
                    }
                }
            }
        } else {
            // If there is no $, no other checks are necessary
            *TASK_INPUT.write() = input_trimmed.to_string();
        }
    }
}

fn has_max_two_decimals(input: &str) -> bool {
    let parts: Vec<&str> = input.split('.').collect();
    match parts.len() {
        1 => true,
        2 => {
            let decimal_part = parts[1];
            decimal_part.len() <= 2
        }
        _ => false,
    }
}
