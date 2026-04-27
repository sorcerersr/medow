use crate::{downloads, navigate, View, APP_STATE, CONFIG};
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
fn header_bar() -> Element {
    let queue = use_memo(|| downloads::get_selected_items());
    let stats = use_memo(move || {
        let items = queue();
        let total = items.len();
        let active = items.iter().filter(|i| matches!(i.status, downloads::DownloadStatus::Downloading { .. })).count();
        let complete = items.iter().filter(|i| matches!(i.status, downloads::DownloadStatus::Complete)).count();
        let failed = items.iter().filter(|i| matches!(i.status, downloads::DownloadStatus::Failed(_) | downloads::DownloadStatus::Cancelled | downloads::DownloadStatus::Paused { .. })).count();
        (total, active, complete, failed)
    });

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
        div { class: "download-stats",
            span { "Total: {stats().0} | Active: {stats().1} | Complete: {stats().2} | Failed/Stopped: {stats().3}" }
            button {
                class: "button",
                onclick: move |_| downloads::open_download_dir(),
                title: "Open download directory",
                "📁 Open Dir"
            }
        }
    }
}

#[component]
fn download_row(item: downloads::DownloadItem) -> Element {
    let filename = item.filename.clone();

    // Format file sizes
    let format_size = |bytes: u64| -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    // Format rate
    let format_rate = |bps: f64| -> String {
        if bps < 1024.0 {
            format!("{:.0} B/s", bps)
        } else if bps < 1024.0 * 1024.0 {
            format!("{:.1} KB/s", bps / 1024.0)
        } else {
            format!("{:.1} MB/s", bps / (1024.0 * 1024.0))
        }
    };

    // Format ETA
    let format_eta = |seconds: f64| -> String {
        if seconds < 60.0 {
            format!("{:.0}s", seconds)
        } else if seconds < 3600.0 {
            format!("{:.0}m {:.0}s", seconds / 60.0, seconds % 60.0)
        } else {
            format!("{:.0}h {:.0}m", seconds / 3600.0, (seconds % 3600.0) / 60.0)
        }
    };

    // Build tooltip content
    let tooltip_parts: Vec<String> = {
        let mut parts = Vec::new();
        if let Some(total) = item.total_size {
            parts.push(format!("Size: {}/{}", format_size(item.downloaded_bytes), format_size(total)));
        } else {
            parts.push(format!("Downloaded: {}", format_size(item.downloaded_bytes)));
        }
        if item.current_rate > 0.0 {
            parts.push(format!("Rate: {}", format_rate(item.current_rate)));
        }
        if let Some(eta) = item.est_time_remaining {
            if eta.is_finite() && eta > 0.0 {
                parts.push(format!("ETA: {}", format_eta(eta)));
            }
        }
        parts
    };
    let tooltip = if tooltip_parts.is_empty() {
        String::new()
    } else {
        tooltip_parts.join(" | ")
    };

    let is_downloading = matches!(item.status, downloads::DownloadStatus::Downloading { .. });
    let is_paused = matches!(item.status, downloads::DownloadStatus::Paused { .. });
    let is_failed = matches!(item.status, downloads::DownloadStatus::Failed(_));
    let is_cancelled = matches!(item.status, downloads::DownloadStatus::Cancelled);
    let is_retry = matches!(item.status, downloads::DownloadStatus::Retry);
    let is_idle = matches!(item.status, downloads::DownloadStatus::Idle);

    // Store filename in signal for closures
    let fname = use_signal(|| filename.clone());
    let tooltip_ref = &tooltip;

    rsx! {
        tr {
            class: "download-row",
            title: "{tooltip_ref}",
            td { class: "download-title", "{item.title}" }
            td { class: "download-filename", "{item.filename}" }
            td {
                span {
                    class: match &item.status {
                        downloads::DownloadStatus::Idle => "status-idle",
                        downloads::DownloadStatus::Downloading { .. } => "status-downloading",
                        downloads::DownloadStatus::Paused { .. } => "status-paused",
                        downloads::DownloadStatus::Complete => "status-complete",
                        downloads::DownloadStatus::Failed(_) => "status-failed",
                        downloads::DownloadStatus::Cancelled => "status-cancelled",
                        downloads::DownloadStatus::Retry => "status-retry",
                    },
                    match &item.status {
                        downloads::DownloadStatus::Idle => String::from("Waiting..."),
                        downloads::DownloadStatus::Downloading { progress } => {
                            format!("Downloading {:.1}%", progress)
                        }
                        downloads::DownloadStatus::Paused { progress } => {
                            format!("Paused {:.1}%", progress)
                        }
                        downloads::DownloadStatus::Complete => String::from("Complete"),
                        downloads::DownloadStatus::Failed(msg) => {
                            format!("Failed: {}", msg)
                        }
                        downloads::DownloadStatus::Cancelled => String::from("Cancelled"),
                        downloads::DownloadStatus::Retry => String::from("Retrying..."),
                    }
                }
            }
            td { class: "download-controls",
                if is_downloading {
                    button {
                        class: "button small",
                        onclick: move |_| downloads::pause_download(&fname()),
                        title: "Pause download",
                        "⏸"
                    }
                }
                if is_paused {
                    button {
                        class: "button small",
                        onclick: move |_| downloads::resume_download(&fname()),
                        title: "Resume download",
                        "▶"
                    }
                }
                if is_downloading || is_paused {
                    button {
                        class: "button small",
                        onclick: move |_| downloads::cancel_download(&fname()),
                        title: "Cancel download",
                        "⏹"
                    }
                }
                if is_failed || is_cancelled {
                    button {
                        class: "button small",
                        onclick: move |_| downloads::retry_download(&fname()),
                        title: "Retry download",
                        "🔄"
                    }
                }
                if is_idle || is_retry {
                    button {
                        class: "button small",
                        onclick: move |_| {
                            let f = fname();
                            let mut queue = DOWNLOAD_QUEUE.write();
                            if let Some(itm) = queue.iter_mut().find(|i| i.filename == f) {
                                itm.status = downloads::DownloadStatus::Idle;
                            }
                            drop(queue);
                            spawn(async move {
                                downloads::start_downloads().await;
                            });
                        },
                        title: "Start download",
                        "▶"
                    }
                }
            }
        }
    }
}

// Re-export DOWNLOAD_QUEUE for use in download_row
static DOWNLOAD_QUEUE: GlobalSignal<Vec<downloads::DownloadItem>> = Signal::global(Vec::new);

#[component]
pub fn downloads_view() -> Element {
    // Throttled refresh: read queue every second
    let mut queue = use_signal(|| downloads::get_selected_items());
    let is_empty = queue().is_empty();

    // Start downloads when there are items with Idle status
    use_effect(move || {
        let has_idle = queue().iter().any(|item| {
            matches!(item.status, downloads::DownloadStatus::Idle | downloads::DownloadStatus::Retry)
        });
        if has_idle {
            // Read all global signals before spawning (spawn runs on tokio thread)
            let items = downloads::get_selected_items();
            let dest_dir = crate::config::get_download_dir();
            let max_concurrent = CONFIG.read().max_concurrent_downloads;
            let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent as usize));

            spawn(async move {
                downloads::start_downloads_impl(items, dest_dir, semaphore).await;
            });
        }
    });

    // Throttle UI refresh to 1 second
    use_effect(move || {
        let mut queue_write = queue.write();
        *queue_write = downloads::get_selected_items();
        drop(queue_write);

        spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let mut queue_write = queue.write();
                *queue_write = downloads::get_selected_items();
                drop(queue_write);
            }
        });
    });

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
                                th { scope: "col", "Controls" }
                            }
                        }
                        tbody {
                            for item in queue().iter().cloned() {
                                download_row { item }
                            }
                        }
                    }
                }

                // Clear buttons at bottom right
                div { class: "download-actions",
                    button {
                        class: "button",
                        onclick: move |_| downloads::clear_download_queue(),
                        "Clear All"
                    }
                    button {
                        class: "button",
                        onclick: move |_| downloads::clear_finished_downloads(),
                        "Clear All Finished"
                    }
                }
            }
        }
    }
}
