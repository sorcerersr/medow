use std::collections::HashMap;

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

/// Sanitize a string for use in a filename
pub fn sanitize_filename(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Extract file extension from URL, defaulting to "mp4"
pub fn extract_extension(url: &str) -> &str {
    if let Some(filename) = url.rsplit('/').next() {
        let name = filename.split('?').next().unwrap_or(filename);
        if let Some(ext) = name.rsplit('.').next() {
            if !ext.is_empty() && ext != name {
                return ext;
            }
        }
    }
    "mp4"
}

#[component]
fn media_table(pagination: Signal<Pagination>) -> Element {
    // Memo: total selected across all pages
    let total_selected_count = use_memo(|| APP_STATE.read().selected_items.len());

    // Memo: header checkbox state based on current page items
    let header_state = use_memo({
        let pagination = pagination;
        move || {
            let all_selected = pagination.read().items.iter().all(|i| i.selected);
            let any_selected = pagination.read().items.iter().any(|i| i.selected);
            let none_selected = !any_selected;
            let indeterminate = !all_selected && !none_selected;
            (all_selected, indeterminate)
        }
    });

    rsx! {
        table {
            thead {
                tr {
                    th {
                        scope: "col",
                        input {
                            r#type: "checkbox",
                            checked: header_state().0,
                            class: if header_state().1 { "indeterminate" } else { "" },
                            oninput: move |_| {
                                // Determine new state: if all selected → deselect all, otherwise select all
                                let all_selected = pagination.read().items.iter().all(|item| item.selected);
                                let new_state = !all_selected;

                                // Update items
                                for item in pagination.write().items.iter_mut() {
                                    item.selected = new_state;
                                }

                                // Sync with global selection set
                                let items_info: Vec<(String, String, String)> = pagination.read().items.iter().map(|i| (i.video_url.clone(), i.title.clone(), i.topic.clone())).collect();
                                {
                                    let mut state = APP_STATE.write();
                                    for (url, title, topic) in items_info {
                                        if new_state {
                                            state.selected_items.insert(url, (title, topic));
                                        } else {
                                            state.selected_items.remove(&url);
                                        }
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
                // Render each item as a table row
                for (index, item) in pagination.read().items.iter().enumerate() {
                    tr {
                        td {
                            input {
                                r#type: "checkbox",
                                checked: item.selected,
                                oninput: move |_| {
                                    let video_url = pagination.read().items[index].video_url.clone();
                                    let was_selected = pagination.read().items[index].selected;

                                    // Toggle item selection
                                    pagination.write().items[index].selected = !was_selected;

                                    // Sync with global selection set
                                    let item_title = pagination.read().items[index].title.clone();
                                    let item_topic = pagination.read().items[index].topic.clone();
                                    {
                                        let mut state = APP_STATE.write();
                                        if was_selected {
                                            state.selected_items.remove(&video_url);
                                        } else {
                                            state.selected_items.insert(video_url, (item_title, item_topic));
                                        }
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
        // Download button below the table
        if total_selected_count() > 0 {
            div {
                class: "download-bar",
                p {
                    class: "download-info",
                    "Selected: {total_selected_count()} item",
                    if total_selected_count() != 1 { "s" }
                }
                button {
                    class: "button download-button",
                    onclick: move |_| {
                        // Collect all selected items with full info from global state
                        let selected_items: HashMap<String, (String, String)> = APP_STATE.read().selected_items.clone();

                        let max_concurrent = 5;
                        // Collect commands into a vector for stable ordering
                        let mut commands: Vec<String> = Vec::new();
                        for (video_url, (title, topic)) in &selected_items {
                            let title_part = sanitize_filename(title);
                            let topic_part = sanitize_filename(topic);
                            let ext = extract_extension(video_url);
                            let filename = format!("{}_{}.{}", title_part, topic_part, ext);
                            commands.push(format!(
                                "wget -t 0 -c -O \"{}\" \"{}\" &",
                                filename, video_url
                            ));
                        }

                        // Emit in blocks of max_concurrent, with `wait` between blocks
                        let mut script_lines: Vec<String> = vec![
                            "#!/bin/bash".to_string(),
                            "# Medow Download Script".to_string(),
                            format!(
                                "# {} files to download ({} per batch)",
                                commands.len(),
                                max_concurrent
                            ),
                            String::new(),
                        ];

                        let total_batches = commands.len().div_ceil(max_concurrent);
                        for (batch_num, chunk) in commands.chunks(max_concurrent).enumerate() {
                            let batch = batch_num + 1;
                            script_lines.push(format!(
                                "echo \"Starting batch {}/{}...\"",
                                batch, total_batches
                            ));
                            script_lines.extend(chunk.iter().cloned());
                            script_lines.push(String::new());
                            script_lines.push("# Wait for this batch to complete".to_string());
                            script_lines.push("wait".to_string());
                            script_lines.push(String::new());
                        }

                        script_lines.push("echo \"All downloads complete.\"".to_string());

                        let script = script_lines.join("\n");
                        APP_STATE.write().download_script = Some(script);
                        navigate(View::Download);
                    },
                    "📥 Generate Download Script"
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
