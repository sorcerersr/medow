use crate::{navigate, toast, View, APP_STATE};
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
    let script = use_memo(|| APP_STATE.read().download_script.clone());

    rsx! {
        header_bar {}
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                h1 { "Download Script" }
                if let Some(script_content) = script() {
                    div {
                        class: "download-actions",
                        button {
                            class: "button copy-button",
                            onclick: move |_| {
                                let text = APP_STATE.read().download_script.clone();
                                if let Some(text) = text {
                                    match arboard::Clipboard::new() {
                                        Ok(mut clipboard) => {
                                            if let Err(e) = clipboard.set_text(&text) {
                                                toast::show_toast(&format!("Failed to copy: {}", e), toast::ToastType::Error(e.to_string()));
                                            } else {
                                                toast::show_toast("Script copied to clipboard!", toast::ToastType::Success);
                                            }
                                        }
                                        Err(e) => {
                                            toast::show_toast(&format!("Clipboard not available: {}", e), toast::ToastType::Error(e.to_string()));
                                        }
                                    }
                                }
                            },
                            "📋 Copy to Clipboard"
                        }
                        button {
                            class: "button",
                            onclick: move |_| navigate(View::Search),
                            "← Back to Search"
                        }
                    }
                    pre {
                        class: "download-script",
                        code { {script_content} }
                    }
                } else {
                    p { "No download script generated. Go to Search and select items to download." }
                    button {
                        class: "button",
                        onclick: move |_| navigate(View::Search),
                        "← Back to Search"
                    }
                }
            }
        }
    }
}
