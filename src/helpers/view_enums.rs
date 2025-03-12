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

use serde::{Deserialize, Serialize};

use crate::loc;
use crate::localization::Localization;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FurView {
    Shortcuts,
    Timer,
    Todo,
    Report,
    Settings,
}

impl FurView {
    pub const ALL: [FurView; 5] = [
        FurView::Shortcuts,
        FurView::Timer,
        FurView::Todo,
        FurView::Report,
        FurView::Settings,
    ];
}

impl std::fmt::Display for FurView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FurView::Shortcuts => loc!("shortcuts"),
                FurView::Timer => loc!("timer"),
                FurView::Todo => loc!("todo"),
                FurView::Report => loc!("report"),
                FurView::Settings => loc!("settings"),
            }
        )
    }
}
