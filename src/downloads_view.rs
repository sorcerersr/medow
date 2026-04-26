use crate::{downloads, navigate, View, APP_STATE};
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
    let queue = use_memo(|| downloads::get_selected_items());
    let is_empty = queue().is_empty();

    // Start downloads when there are items with Idle status
    use_effect(move || {
        let has_idle = queue().iter().any(|item| {
            matches!(item.status, downloads::DownloadStatus::Idle)
        });
        if has_idle {
            spawn(async move {
                downloads::start_downloads().await;
            });
        }
    });

    // Pre-compute status for all items
    let status_data: Vec<(String, String)> = queue()
        .iter()
        .map(|item| {
            let status_text = match &item.status {
                downloads::DownloadStatus::Idle => String::from("Waiting..."),
                downloads::DownloadStatus::Downloading { progress } => {
                    format!("Downloading {:.1}%", progress)
                }
                downloads::DownloadStatus::Complete => String::from("Complete"),
                downloads::DownloadStatus::Failed(msg) => {
                    format!("Failed: {}", msg)
                }
            };
            let status_class = match &item.status {
                downloads::DownloadStatus::Idle => String::from("status-idle"),
                downloads::DownloadStatus::Downloading { .. } => String::from("status-downloading"),
                downloads::DownloadStatus::Complete => String::from("status-complete"),
                downloads::DownloadStatus::Failed(_) => String::from("status-failed"),
            };
            (status_text, status_class)
        })
        .collect();

    rsx! {
        header_bar {}
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                h1 { "Downloads" }

                if is_empty {
                    p { "No downloads in queue." }
                } else {
                    table {
                        thead {
                            tr {
                                th { scope: "col", "Title" }
                                th { scope: "col", "Filename" }
                                th { scope: "col", "Status" }
                            }
                        }
                        tbody {
                            for (item, (status_text, status_class)) in queue().iter().zip(status_data.iter()) {
                                tr {
                                    td { "{item.title}" }
                                    td { "{item.filename}" }
                                    td { span { class: "{status_class}", "{status_text}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
