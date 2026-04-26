use crate::{navigate, APP_STATE, View};
use dioxus::prelude::*;

#[component]
fn header_bar() -> Element {
    rsx! {
        header {
            class: "sticky-header",
            article {
                padding_bottom: 0,
                padding_top: 0,
                nav {
                    ul {
                        if APP_STATE.read().view != View::Search {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Search),
                                    "Search"
                                }
                            }
                        }
                        if APP_STATE.read().view != View::Settings {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Settings),
                                    "Settings"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn downloads_view() -> Element {
    rsx! {
        header_bar {}
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                h1 { "Downloads View" }
            }
        }
    }
}
