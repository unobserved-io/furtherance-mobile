mod models {
    pub mod fur_settings;
    pub mod fur_task;
    pub mod fur_task_group;
}
mod helpers {
    pub mod actions;
    pub mod database;
    pub mod formatters;
    pub mod tasks;
    pub mod view_enums;
    pub mod views {
        pub mod task_input;
        pub mod timer;
    }
}
mod views {
    pub mod timer_view;
}
mod constants;
mod localization;
mod state;

use constants::{FAVICON, MAIN_CSS};
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsBookmark, BsCheck2Circle, BsGear, BsHourglassSplit},
    Icon, IconShape,
};
use helpers::{database::db_init, formatters};
use views::timer_view::TimerView;

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    state::use_task_history_provider();
    let mut active_tab = use_signal(|| NavTab::Timer);
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        // TopNav {}

        div { id: "page-content",
            match *active_tab.read() {
                NavTab::Timer => rsx! {
                    TimerView {}
                },
                NavTab::Todos => rsx! {
                    TimerView {}
                },
                NavTab::Shortcuts => rsx! {
                    TimerView {}
                },
                NavTab::Settings => rsx! {
                    TimerView {}
                },
            }
        }

        BottomNav {
            active_tab: *active_tab.read(),
            on_tab_change: move |tab| active_tab.set(tab),
        }
    }
}

#[component]
pub fn TopNav() -> Element {
    // TODO: Redo this for a better way to protect the safe area
    rsx! {
        div { id: "navbar" }
    }
}

#[component]
pub fn BottomNav(active_tab: NavTab, on_tab_change: EventHandler<NavTab>) -> Element {
    rsx! {
        div { class: "bottom-nav",
            NavItem {
                icon: BsHourglassSplit,
                label: "Timer",
                active: active_tab == NavTab::Timer,
                onclick: move |_| on_tab_change.call(NavTab::Timer),
            }
            NavItem {
                icon: BsCheck2Circle,
                label: "Todos",
                active: active_tab == NavTab::Todos,
                onclick: move |_| on_tab_change.call(NavTab::Todos),
            }
            NavItem {
                icon: BsBookmark,
                label: "Shortcuts",
                active: active_tab == NavTab::Shortcuts,
                onclick: move |_| on_tab_change.call(NavTab::Shortcuts),
            }
            NavItem {
                icon: BsGear,
                label: "Settings",
                active: active_tab == NavTab::Settings,
                onclick: move |_| on_tab_change.call(NavTab::Settings),
            }
        }
    }
}

#[component]
fn NavItem<I: IconShape + Clone + PartialEq + 'static>(
    icon: I,
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if active {
        "nav-item active"
    } else {
        "nav-item"
    };

    rsx! {
        div { class: "{class}", onclick: move |e| onclick.call(e),
            Icon { icon, width: 25, height: 25 }
            span { class: "nav-label", "{label}" }
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum NavTab {
    Timer,
    Todos,
    Shortcuts,
    Settings,
}
