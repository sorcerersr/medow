use dioxus::prelude::*;
use futures::StreamExt;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::CONFIG;

#[derive(Clone, PartialEq, Debug)]
pub enum DownloadStatus {
    Idle,
    Downloading { progress: f64 },
    Complete,
    Failed(String),
}

#[derive(Clone, PartialEq, Debug)]
pub struct DownloadItem {
    pub title: String,
    pub url: String,
    pub filename: String,
    pub status: DownloadStatus,
}

// Global selection state — persists across pagination
static SELECTED_TITLES: GlobalSignal<std::collections::HashSet<String>> =
    Signal::global(std::collections::HashSet::new);

// Global download queue
static DOWNLOAD_QUEUE: GlobalSignal<Vec<DownloadItem>> = Signal::global(Vec::new);

/// Get all selected titles
#[allow(dead_code)]
pub fn get_selected_titles() -> std::collections::HashSet<String> {
    SELECTED_TITLES.read().clone()
}

/// Check if a title is selected
pub fn is_selected(title: &str) -> bool {
    SELECTED_TITLES.read().contains(title)
}

/// Toggle selection of a title
pub fn toggle_selection(title: String) {
    let mut titles = SELECTED_TITLES.write();
    if titles.contains(&title) {
        titles.remove(&title);
    } else {
        titles.insert(title);
    }
}

/// Clear all selections
pub fn clear_selections() {
    SELECTED_TITLES.write().clear();
}

/// Get all currently selected items with their URLs
pub fn get_selected_items() -> Vec<DownloadItem> {
    DOWNLOAD_QUEUE.read().clone()
}

/// Add items to the download queue
pub fn add_to_download_queue(items: Vec<DownloadItem>) {
    let mut queue = DOWNLOAD_QUEUE.write();
    queue.clear();
    queue.extend(items);
}

/// Clear the download queue
#[allow(dead_code)]
pub fn clear_download_queue() {
    DOWNLOAD_QUEUE.write().clear();
}

/// Get download queue count
#[allow(dead_code)]
pub fn download_queue_len() -> usize {
    DOWNLOAD_QUEUE.read().len()
}

/// Generate filename from title and URL
pub fn generate_filename(title: &str, url: &str) -> String {
    let filename = title.replace(' ', "_");
    // Extract extension from URL
    let ext = url
        .split('?')
        .next()
        .and_then(|u| u.rsplit_once('/'))
        .map(|(_, path)| {
            path.rsplit_once('.')
                .map(|(_, ext)| ext.to_string())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    if ext.is_empty() {
        filename
    } else {
        format!("{}.{}", filename, ext)
    }
}

/// Download a single file asynchronously
pub async fn download_file(url: String, dest_dir: PathBuf, filename: String) -> DownloadStatus {
    // Ensure destination directory exists
    if let Err(e) = tokio::fs::create_dir_all(&dest_dir).await {
        eprintln!("[download] Failed to create dest dir: {e}");
        return DownloadStatus::Failed(format!("Failed to create directory: {e}"));
    }

    let dest_path = dest_dir.join(&filename);

    // Fetch the file
    let client = reqwest::Client::new();
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[download] Failed to fetch {url}: {e}");
            return DownloadStatus::Failed(format!("Request failed: {e}"));
        }
    };

    if !response.status().is_success() {
        eprintln!("[download] HTTP error: {} for {url}", response.status());
        return DownloadStatus::Failed(format!("HTTP {}", response.status()));
    }

    let total_size = response.content_length();
    let mut file = match File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[download] Failed to create file: {e}");
            return DownloadStatus::Failed(format!("File create failed: {e}"));
        }
    };

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[download] Stream error: {e}");
                return DownloadStatus::Failed(format!("Stream error: {e}"));
            }
        };

        if let Err(e) = file.write_all(&bytes).await {
            eprintln!("[download] Write error: {e}");
            return DownloadStatus::Failed(format!("Write error: {e}"));
        }

        downloaded += bytes.len() as u64;

        // Update progress
        let progress = if let Some(total) = total_size {
            (downloaded as f64 / total as f64 * 100.0).min(100.0)
        } else {
            // No content-length, estimate based on some heuristic or just show indeterminate
            // For simplicity, show 50% until we know more
            50.0
        };

        // Update the download queue with progress
        let mut queue = DOWNLOAD_QUEUE.write();
        if let Some(item) = queue.iter_mut().find(|i| i.filename == filename) {
            item.status = DownloadStatus::Downloading { progress };
        }
        drop(queue);
    }

    // Mark as complete
    let mut queue = DOWNLOAD_QUEUE.write();
    if let Some(item) = queue.iter_mut().find(|i| i.filename == filename) {
        item.status = DownloadStatus::Complete;
    }
    drop(queue);

    DownloadStatus::Complete
}

/// Start all downloads in the queue
pub async fn start_downloads() {
    let items = DOWNLOAD_QUEUE.read().clone();
    let dest_dir: PathBuf = CONFIG.read().default_download_dir.clone().into();

    if dest_dir.as_os_str().is_empty() {
        eprintln!("[download] No download directory configured");
        return;
    }

    for item in items {
        let url = item.url.clone();
        let filename = item.filename.clone();
        let dest = dest_dir.clone();

        // Spawn individual download tasks (non-blocking, tied to downloads_view lifecycle)
        spawn(async move {
            download_file(url, dest, filename).await;
        });
    }
}
