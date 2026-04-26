use crate::{navigate, pagination::Pagination, search_logic, View, APP_STATE};
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
    // Create a signal for the header checkbox state
    let mut header_selected = use_signal(|| false);
    rsx! {
        table {
            thead {
                tr {
                    th {
                        scope: "col",
                        input {
                            r#type: "checkbox",
                            checked: header_selected(),
                            oninput: move |e| {
                                let checked = e.checked();
                                header_selected.set(checked);
                                // Update all items' selected state
                                for item in pagination.write().items.iter_mut() {
                                    item.selected = checked;
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
                // Render each item as a table row
                for (index, item) in pagination.read().items.iter().enumerate() {
                    tr {
                        td {
                            input {
                                r#type: "checkbox",
                                checked: item.selected,
                                oninput: move |e| {
                                    let checked = e.checked();
                                    // Update the specific item by index
                                    pagination.write().items[index].selected = checked;

                                    // Update header checkbox based on all items
                                    let all_selected = pagination.read().items.iter().all(|item| item.selected);
                                    let any_selected = pagination.read().items.iter().any(|item| item.selected);

                                    if all_selected {
                                        header_selected.set(true);
                                    } else if !any_selected {
                                        header_selected.set(false);
                                    }
                                }
                            }

                        }
                        td { "{item.title}" }
                        td { "{item.topic}" }
                        td { "{item.timestamp}" }
                        td { "{item.duration}" }
                        td { "{item.quality}" }
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
                        {pagination.read().visible_pages().iter().map(|page| {
                            let page = *page;
                            let is_current = page == pagination.read().current_page();
                            let offset = (page - 1) * pagination.read().page_size();
                            rsx! {
                                button {
                                    class: if is_current { "button current-page" } else { "button" },
                                    onclick: move |_| async move {
                                        search_logic::perform_search(pagination, searchstring(), offset).await;
                                    },
                                    "{page}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
