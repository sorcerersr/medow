use crate::{downloads, navigate, pagination::Pagination, search_logic, View, APP_STATE};
use dioxus::prelude::*;

#[component]
pub fn header_bar(pagination: Signal<Pagination>, mut searchstring: Signal<String>) -> Element {
    rsx! {
        header {
            class: "sticky-header",
            article {
                padding_bottom: 0,
                padding_top: 0,
                nav {
                    ul {
                        li {
                            input {
                                r#type: "text",
                                placeholder: "Search...",
                                class: "input search-input",
                                oninput: move |event_data| {
                                    searchstring.set(event_data.value());
                                },
                                onkeydown: move |event_data| async move {
                                    if event_data.key() == Key::Enter {
                                        search_logic::perform_search(pagination, searchstring(), 0).await;
                                    }
                                }
                            }
                        }
                        li {
                            button {
                                class: "button",
                                onclick: move |_| {
                                    search_logic::perform_search(pagination, searchstring(), 0)
                                },
                                "Search",
                            }
                        }
                    }
                    ul {
                        if APP_STATE.read().view != View::Download {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Download),
                                    "Downloads"
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
fn media_table(pagination: Signal<Pagination>) -> Element {
    let header_checked = use_memo(move || {
        let items = pagination.read().items.clone();
        if items.is_empty() {
            return false;
        }
        items.iter().all(|item| downloads::is_selected(&item.title))
    });

    // Pre-compute item data for RSX loop (must be outside rsx! block)
    let item_data: Vec<_> = pagination
        .read()
        .items
        .iter()
        .map(|item| {
            (
                item.title.clone(),
                downloads::is_selected(&item.title),
                item.topic.clone(),
                item.timestamp.clone(),
                item.duration.clone(),
                item.quality.clone(),
            )
        })
        .collect();

    rsx! {
        table {
            thead {
                tr {
                    th {
                        scope: "col",
                        input {
                            r#type: "checkbox",
                            checked: header_checked(),
                            onchange: move |e| {
                                let checked = e.checked();
                                let items: Vec<_> = pagination.read().items.clone();
                                for item in items {
                                    if checked {
                                        downloads::toggle_selection(item.title.clone());
                                    } else {
                                        downloads::toggle_selection(item.title);
                                    }
                                }
                            }
                        }
                    }
                    th { scope: "col", "title" }
                    th { scope: "col", "topic" }
                    th { scope: "col", "timestamp" }
                    th { scope: "col", "duration" }
                    th { scope: "col", "quality" }
                }
            }
            tbody {
                for (title, checked, topic, timestamp, duration, quality) in item_data {
                    tr {
                        td {
                            input {
                                r#type: "checkbox",
                                checked: checked,
                                onchange: move |e| {
                                    let checked = e.checked();
                                    let title_for_toggle = title.clone();
                                    if checked {
                                        downloads::toggle_selection(title_for_toggle);
                                    } else {
                                        downloads::toggle_selection(title_for_toggle);
                                    }
                                }
                            }
                        }
                        td { "{title}" }
                        td { "{topic}" }
                        td { "{timestamp}" }
                        td { "{duration}" }
                        td { "{quality}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn search_view() -> Element {
    let pagination = use_signal(Pagination::new);
    let searchstring = use_signal(String::new);

    rsx! {
        header_bar { pagination, searchstring }
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                if APP_STATE.read().is_loading {
                    progress {  }
                } else {
                    media_table { pagination }
                }
            } // article

            // Download action buttons - right-aligned
            if !APP_STATE.read().is_loading {
                div {
                    class: "download-buttons",
                    button {
                        class: "button",
                        onclick: move |_| {
                            // Collect selected items from current page
                            let selected_items: Vec<downloads::DownloadItem> = pagination
                                .read()
                                .items
                                .iter()
                                .filter(|item| downloads::is_selected(&item.title))
                                .map(|item| downloads::DownloadItem {
                                    title: item.title.clone(),
                                    url: item.video_url.clone(),
                                    filename: downloads::generate_filename(&item.title, &item.video_url),
                                    status: downloads::DownloadStatus::Idle,
                                })
                                .collect();

                            if !selected_items.is_empty() {
                                downloads::add_to_download_queue(selected_items);
                                navigate(View::Download);
                            }
                        },
                        "Download",
                    },
                    button {
                        class: "button",
                        onclick: move |_| {
                            downloads::clear_selections();
                        },
                        "Reset",
                    },
                }
            }
        }
        footer {
            class: "sticky-footer",
            article {
                padding_bottom: 0,
                padding_top: 0,
                nav {
                    ul {
                        li { "{pagination.read().info()}" }
                        if pagination.read().has_previous_page() {
                            button {
                                class: "button",
                                onclick: move |_| async move {
                                    let offset = pagination.read().first_offset();
                                    if let Some(offset) = offset {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    }
                                },
                                "« First"
                            }
                        } else {
                            button { class: "button", disabled: true, "« First" }
                        }
                        if pagination.read().has_previous_page() {
                            button {
                                class: "button",
                                onclick: move |_| async move {
                                    let offset = pagination.read().previous_offset();
                                    if let Some(offset) = offset {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    }
                                },
                                "← Prev"
                            }
                        } else {
                            button { class: "button", disabled: true, "← Prev" }
                        }
                    }
                    ul {
                        {pagination.read().visible_pages().iter().map(|page| {
                            let page = *page;
                            let is_current = page == pagination.read().current_page();
                            let offset = (page - 1) * pagination.read().page_size();
                            rsx! {
                                button {
                                    class: "button",
                                    disabled: is_current,
                                    onclick: move |_| async move {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    },
                                    "{page}"
                                }
                            }
                        })}
                    }
                    ul {
                        if pagination.read().has_next_page() {
                            button {
                                class: "button",
                                onclick: move |_| async move {
                                    let offset = pagination.read().next_offset();
                                    if let Some(offset) = offset {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    }
                                },
                                "Next →"
                            }
                        } else {
                            button { class: "button", disabled: true, "Next →" }
                        }
                        if pagination.read().has_next_page() {
                            button {
                                class: "button",
                                onclick: move |_| async move {
                                    let offset = pagination.read().last_offset();
                                    if let Some(offset) = offset {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    }
                                },
                                "Last »"
                            }
                        } else {
                            button { class: "button", disabled: true, "Last »" }
                        }
                    }
                }
            }
        }
    }
}
