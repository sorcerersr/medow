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
                        margin_left: "auto",
                        if APP_STATE.read().view != View::Search {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Search),
                                    "Search"
                                }
                            }
                        }
                        if APP_STATE.read().view != View::Download {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Download),
                                    "Downloads"
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
pub fn settings_view() -> Element {
    rsx! {
        header_bar {}
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                h1 { "Settings View" }
            }
        }
    }
}
