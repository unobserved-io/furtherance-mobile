mod models {
    pub mod fur_settings;
    pub mod fur_task;
    pub mod fur_task_group;
    pub mod fur_todo;
}
pub mod database {
    pub mod init;
    pub mod tasks;
    pub mod todos;
}
mod helpers {
    pub mod actions;
    pub mod formatters;
    pub mod tasks;
    pub mod view_enums;
    pub mod views {
        pub mod task_input;
        pub mod timer;
        pub mod todos;
    }
    pub mod todos;
}
mod views {
    pub mod timer_view;
    pub mod todos_view;
}
mod constants;
mod localization;
mod state;

use constants::{FAVICON, MAIN_CSS, TIMER_CSS, TODO_CSS};
use database::init::db_init;
use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::bs_icons::{BsBookmark, BsCheck2Circle, BsGear, BsHourglassSplit},
    Icon, IconShape,
};
use helpers::{formatters, views::timer::ensure_timer_running};
use state::ACTIVE_TAB;
use views::{timer_view::TimerView, todos_view::TodosView};

#[derive(PartialEq, Copy, Clone)]
pub enum NavTab {
    Timer,
    Todos,
    Shortcuts,
    Settings,
}

fn main() {
    db_init().expect("Failed to read or create database");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    state::use_task_history_provider();
    state::use_all_todos_provider();
    ensure_timer_running();
    // let mut active_tab = use_signal(|| NavTab::Timer);
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: MAIN_CSS }
        document::Stylesheet { href: TIMER_CSS }
        document::Stylesheet { href: TODO_CSS }
        // TopNav {}

        div { id: "page-content",
            match ACTIVE_TAB.cloned() {
                NavTab::Timer => rsx! {
                    TimerView {}
                },
                NavTab::Todos => rsx! {
                    TodosView {}
                },
                NavTab::Shortcuts => rsx! {
                    TimerView {}
                },
                NavTab::Settings => rsx! {
                    TimerView {}
                },
            }
        }

        BottomNav {}
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
pub fn BottomNav() -> Element {
    let active_tab = ACTIVE_TAB.cloned();
    rsx! {
        div { class: "bottom-nav",
            NavItem {
                icon: BsHourglassSplit,
                label: "Timer",
                active: active_tab == NavTab::Timer,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Timer,
            }
            NavItem {
                icon: BsCheck2Circle,
                label: "Todos",
                active: active_tab == NavTab::Todos,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Todos,
            }
            NavItem {
                icon: BsBookmark,
                label: "Shortcuts",
                active: active_tab == NavTab::Shortcuts,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Shortcuts,
            }
            NavItem {
                icon: BsGear,
                label: "Settings",
                active: active_tab == NavTab::Settings,
                onclick: move |_| *ACTIVE_TAB.write() = NavTab::Settings,
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
