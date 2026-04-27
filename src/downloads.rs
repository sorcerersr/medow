use dioxus::prelude::*;
use futures::StreamExt;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{broadcast, watch, Semaphore};
use tokio::time::interval;

use crate::config;
use crate::CONFIG;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum DownloadStatus {
    Idle,
    Downloading { progress: f64 },
    Paused { progress: f64 },
    Complete,
    Failed(String),
    Cancelled,
    Retry,
}

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct DownloadItem {
    pub title: String,
    pub url: String,
    pub filename: String,
    pub status: DownloadStatus,
    pub total_size: Option<u64>,
    pub downloaded_bytes: u64,
    pub current_rate: f64, // bytes per second
    pub est_time_remaining: Option<f64>, // seconds
}

// Global selection state — persists across pagination
static SELECTED_TITLES: GlobalSignal<std::collections::HashSet<String>> =
    Signal::global(std::collections::HashSet::new);

// Global download queue
pub static DOWNLOAD_QUEUE: GlobalSignal<Vec<DownloadItem>> = Signal::global(Vec::new);

// Download task handles for pause/resume/cancel
static DOWNLOAD_TASKS: GlobalSignal<HashMap<String, dioxus::core::Task>> =
    Signal::global(|| HashMap::new());

// Per-download control channel senders
static DOWNLOAD_CONTROL_SENDERS: GlobalSignal<HashMap<String, broadcast::Sender<DownloadControlMsg>>> =
    Signal::global(|| HashMap::new());

// Semaphore for concurrent download limiting
static DOWNLOAD_SEMAPHORE: GlobalSignal<Arc<Semaphore>> = Signal::global(|| {
    Arc::new(Semaphore::new(3))
});





// Download control messages
#[derive(Clone, Debug)]
pub enum DownloadControlMsg {
    Pause(String),   // filename
    Resume(String),  // filename
    Cancel(String),  // filename
    Retry(String),   // filename
}

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
    DOWNLOAD_TASKS.write().clear();
}

/// Clear all finished downloads (Complete, Failed, Cancelled)
pub fn clear_finished_downloads() {
    let mut queue = DOWNLOAD_QUEUE.write();
    queue.retain(|item| {
        !matches!(
            &item.status,
            DownloadStatus::Complete
                | DownloadStatus::Failed(_)
                | DownloadStatus::Cancelled
        )
    });
    // Remove tasks for cleared items
    let filenames: std::collections::HashSet<String> = queue.iter().map(|i| i.filename.clone()).collect();
    let mut tasks = DOWNLOAD_TASKS.write();
    tasks.retain(|filename, _| filenames.contains(filename));
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

/// Update a download item's status in the queue
fn update_item_status(filename: &str, status: DownloadStatus, extra: DownloadExtra) {
    let mut queue = DOWNLOAD_QUEUE.write();
    if let Some(item) = queue.iter_mut().find(|i| i.filename.as_str() == filename) {
        item.status = status;
        if let DownloadStatus::Downloading { progress } = &item.status {
            item.downloaded_bytes = extra.downloaded_bytes;
            item.current_rate = extra.current_rate;
            item.est_time_remaining = extra.est_time_remaining;
            if let Some(total) = extra.total_size {
                item.total_size = Some(total);
            }
        }
    }
    drop(queue);
}

struct DownloadExtra {
    downloaded_bytes: u64,
    current_rate: f64,
    est_time_remaining: Option<f64>,
    total_size: Option<u64>,
}

/// Download a single file asynchronously with pause/resume/cancel support
async fn download_file(
    url: String,
    dest_dir: PathBuf,
    filename: String,
    semaphore: Arc<Semaphore>,
    mut control_rx: broadcast::Receiver<DownloadControlMsg>,
) {
    // Acquire semaphore slot
    let permit = match semaphore.acquire().await {
        Ok(permit) => permit,
        Err(_) => return,
    };

    // Get current download state for resume
    let resume_offset = DOWNLOAD_QUEUE.read()
        .iter()
        .find(|i| i.filename.as_str() == &filename)
        .map(|i| i.downloaded_bytes)
        .unwrap_or(0);

    let dest_path = dest_dir.join(&filename);

    // Fetch the file with range support for resume
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    if resume_offset > 0 {
        request = request.header("Range", format!("bytes={}-", resume_offset));
    }

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("[download] Failed to fetch {url}: {e}");
            update_item_status(&filename, DownloadStatus::Failed(format!("Request failed: {e}")), DownloadExtra {
                downloaded_bytes: resume_offset,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: None,
            });
            return;
        }
    };

    if !response.status().is_success() {
        if response.status().as_u16() == 416 {
            // Range not satisfiable - file may be complete on disk
            update_item_status(&filename, DownloadStatus::Complete, DownloadExtra {
                downloaded_bytes: resume_offset,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: Some(resume_offset),
            });
            return;
        }
        eprintln!("[download] HTTP error: {} for {url}", response.status());
        update_item_status(&filename, DownloadStatus::Failed(format!("HTTP {}", response.status())), DownloadExtra {
            downloaded_bytes: resume_offset,
            current_rate: 0.0,
            est_time_remaining: None,
            total_size: None,
        });
        return;
    }

    let total_size = response.content_length();
    let mut file = match File::create(&dest_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[download] Failed to create file: {e}");
            update_item_status(&filename, DownloadStatus::Failed(format!("File create failed: {e}")), DownloadExtra {
                downloaded_bytes: resume_offset,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: None,
            });
            return;
        }
    };

    // Seek to resume offset if resuming
    if resume_offset > 0 {
        if let Err(e) = file.seek(tokio::io::SeekFrom::Start(resume_offset)).await {
            eprintln!("[download] Failed to seek to offset {resume_offset}: {e}");
            file = match File::create(&dest_path).await {
                Ok(f) => f,
                Err(e) => {
                    update_item_status(&filename, DownloadStatus::Failed(format!("File create failed: {e}")), DownloadExtra {
                        downloaded_bytes: 0,
                        current_rate: 0.0,
                        est_time_remaining: None,
                        total_size: None,
                    });
                    return;
                }
            };
        }
    }

    let mut downloaded: u64 = resume_offset;
    let mut stream = response.bytes_stream();

    // Rate estimation: sliding window of (instant, bytes)
    let rate_samples: Arc<Mutex<Vec<(Instant, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let mut cancelled = false;

    // Update initial status
    update_item_status(&filename, DownloadStatus::Downloading { progress: 0.0 }, DownloadExtra {
        downloaded_bytes: resume_offset,
        current_rate: 0.0,
        est_time_remaining: None,
        total_size,
    });

    // Spawn UI update task at ~10fps — does NOT block download loop
    let ui_filename = filename.clone();
    let ui_total_size = total_size;
    let ui_downloaded = Arc::new(AtomicU64::new(downloaded));
    let (cancel_tx, mut cancel_rx) = watch::channel(false);
    let (pause_tx, mut pause_rx) = watch::channel(false);
    let ui_rate_samples = Arc::clone(&rate_samples);
    let ui_downloaded_clone = Arc::clone(&ui_downloaded);
    let cancel_tx_clone = cancel_tx.clone();
    let pause_tx_clone = pause_tx.clone();
    let ui_rate_samples_clone = Arc::clone(&ui_rate_samples);

    let _ui_handle = dioxus::prelude::spawn(async move {
        let mut interval = interval(std::time::Duration::from_millis(100));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;

            // Check pause state via watch
            let _ = pause_rx.borrow_and_update();
            let is_paused = *pause_rx.borrow();

            let cur_downloaded = ui_downloaded_clone.load(Ordering::SeqCst);
            let _ = cancel_rx.borrow_and_update();
            let is_cancelled = *cancel_rx.borrow();

            if is_cancelled {
                return;
            }

            if is_paused {
                update_item_status(&ui_filename, DownloadStatus::Paused { progress: calculate_progress(cur_downloaded, ui_total_size) }, DownloadExtra {
                    downloaded_bytes: cur_downloaded,
                    current_rate: 0.0,
                    est_time_remaining: None,
                    total_size: ui_total_size,
                });
                continue;
            }

            let now = Instant::now();
            let current_rate = {
                let samples = ui_rate_samples_clone.lock().unwrap();
                if samples.len() >= 2 {
                    let (first_inst, first_bytes) = samples.first().unwrap();
                    let elapsed = now.duration_since(*first_inst).as_secs_f64();
                    if elapsed > 0.0 {
                        (cur_downloaded - *first_bytes) as f64 / elapsed
                    } else { 0.0 }
                } else { 0.0 }
            };

            let est_time_remaining = if let Some(total) = ui_total_size {
                if current_rate > 0.0 {
                    let remaining = (total as f64 - cur_downloaded as f64).max(0.0);
                    Some(remaining / current_rate)
                } else { None }
            } else { None };

            let progress = calculate_progress(cur_downloaded, ui_total_size);

            update_item_status(&ui_filename, DownloadStatus::Downloading { progress }, DownloadExtra {
                downloaded_bytes: cur_downloaded,
                current_rate,
                est_time_remaining,
                total_size: ui_total_size,
            });
        }
    });

    // Main download loop — no throttle, runs at full speed
    while let Some(chunk) = stream.next().await {
        // Check for control messages (non-blocking)
        loop {
            match control_rx.try_recv() {
                Ok(msg) => match msg {
                    DownloadControlMsg::Pause(_) => {
                        let _ = pause_tx_clone.send(true);
                    }
                    DownloadControlMsg::Resume(_) => {
                        let _ = pause_tx_clone.send(false);
                    }
                    DownloadControlMsg::Cancel(_) => {
                        cancelled = true;
                        let _ = cancel_tx_clone.send(true);
                        break;
                    }
                    DownloadControlMsg::Retry(_) => {}
                },
                Err(broadcast::error::TryRecvError::Empty) => break,
                Err(broadcast::error::TryRecvError::Lagged(_)) => break,
                Err(broadcast::error::TryRecvError::Closed) => return,
            }
        }

        if cancelled {
            break;
        }

        let bytes = match chunk {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[download] Stream error: {e}");
                update_item_status(&filename, DownloadStatus::Failed(format!("Stream error: {e}")), DownloadExtra {
                    downloaded_bytes: downloaded,
                    current_rate: 0.0,
                    est_time_remaining: None,
                    total_size: None,
                });
                return;
            }
        };

        if let Err(e) = file.write_all(&bytes).await {
            eprintln!("[download] Write error: {e}");
            update_item_status(&filename, DownloadStatus::Failed(format!("Write error: {e}")), DownloadExtra {
                downloaded_bytes: downloaded,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: None,
            });
            return;
        }

        downloaded += bytes.len() as u64;

        // Update rate estimation
        let now = Instant::now();
        {
            let mut samples = rate_samples.lock().unwrap();
            samples.push((now, downloaded));
            samples.retain(|(inst, _)| now.duration_since(*inst).as_secs_f64() <= 5.0);
        }

        ui_downloaded.store(downloaded, Ordering::SeqCst);
    }

    drop(permit);

    if cancelled {
        // Remove partial file
        let _ = tokio::fs::remove_file(&dest_path).await;
        update_item_status(&filename, DownloadStatus::Cancelled, DownloadExtra {
            downloaded_bytes: 0,
            current_rate: 0.0,
            est_time_remaining: None,
            total_size: None,
        });
    } else if downloaded == resume_offset && resume_offset > 0 {
        // Resumed download - file was already complete
        update_item_status(&filename, DownloadStatus::Complete, DownloadExtra {
            downloaded_bytes: downloaded,
            current_rate: 0.0,
            est_time_remaining: None,
            total_size: Some(downloaded),
        });
    } else if let Some(total) = total_size {
        if downloaded >= total {
            update_item_status(&filename, DownloadStatus::Complete, DownloadExtra {
                downloaded_bytes: downloaded,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: Some(total),
            });
        } else {
            update_item_status(&filename, DownloadStatus::Failed(format!("Incomplete download: {downloaded}/{total} bytes")), DownloadExtra {
                downloaded_bytes: downloaded,
                current_rate: 0.0,
                est_time_remaining: None,
                total_size: Some(total),
            });
        }
    } else {
        // No total size - mark complete if stream ended
        update_item_status(&filename, DownloadStatus::Complete, DownloadExtra {
            downloaded_bytes: downloaded,
            current_rate: 0.0,
            est_time_remaining: None,
            total_size: None,
        });
    }
}

fn calculate_progress(downloaded: u64, total_size: Option<u64>) -> f64 {
    if let Some(total) = total_size {
        if total > 0 {
            (downloaded as f64 / total as f64 * 100.0).min(100.0)
        } else {
            100.0
        }
    } else {
        50.0 // Indeterminate
    }
}

/// Start all downloads in the queue with concurrency limiting
/// Must be called from within Dioxus runtime (reads global signals)
pub async fn start_downloads() {
    let items = DOWNLOAD_QUEUE.read().clone();
    let dest_dir = config::get_download_dir();
    let max_concurrent = CONFIG.read().max_concurrent_downloads;

    if dest_dir.as_os_str().is_empty() {
        eprintln!("[download] No download directory configured");
        return;
    }

    *DOWNLOAD_SEMAPHORE.write() = Arc::new(Semaphore::new(max_concurrent as usize));
    let semaphore = DOWNLOAD_SEMAPHORE.read().clone();

    start_downloads_impl(items, dest_dir, semaphore).await;
}

/// Internal implementation: runs downloads without reading global signals
pub async fn start_downloads_impl(items: Vec<DownloadItem>, dest_dir: PathBuf, semaphore: Arc<Semaphore>) {
    // Capture Dioxus runtime for spawned tasks
    let runtime = dioxus::core::Runtime::current();

    for item in items {
        if !matches!(item.status, DownloadStatus::Idle | DownloadStatus::Retry) {
            continue;
        }

        let url = item.url.clone();
        let filename = item.filename.clone();
        let dest = dest_dir.clone();
        let sem = semaphore.clone();
        let rt = runtime.clone();

        // Create per-download control channel
        let (control_tx, control_rx) = broadcast::channel::<DownloadControlMsg>(100);

        // Store control sender
        let mut senders = DOWNLOAD_CONTROL_SENDERS.write();
        senders.insert(filename.clone(), control_tx);
        drop(senders);

        // Update status to downloading
        update_item_status(&filename, DownloadStatus::Downloading { progress: 0.0 }, DownloadExtra {
            downloaded_bytes: 0,
            current_rate: 0.0,
            est_time_remaining: None,
            total_size: item.total_size,
        });

        // Spawn download task on Dioxus runtime
        let handle = dioxus::prelude::spawn(async move {

            download_file(url, dest, filename, sem, control_rx).await;
        });

        // Store task handle
        let mut tasks = DOWNLOAD_TASKS.write();
        tasks.insert(item.filename.clone(), handle);
    }
}

/// Pause a download by filename
pub fn pause_download(filename: &str) {
    if let Some(tx) = DOWNLOAD_CONTROL_SENDERS.read().get(filename) {
        let _ = tx.send(DownloadControlMsg::Pause(filename.to_string()));
    }
}

/// Resume a paused download by filename
pub fn resume_download(filename: &str) {
    if let Some(tx) = DOWNLOAD_CONTROL_SENDERS.read().get(filename) {
        let _ = tx.send(DownloadControlMsg::Resume(filename.to_string()));
    }
}

/// Cancel a download by filename
pub fn cancel_download(filename: &str) {
    if let Some(tx) = DOWNLOAD_CONTROL_SENDERS.read().get(filename) {
        let _ = tx.send(DownloadControlMsg::Cancel(filename.to_string()));
    }
    // Remove task handle and sender
    let mut tasks = DOWNLOAD_TASKS.write();
    tasks.remove(filename);
    let mut senders = DOWNLOAD_CONTROL_SENDERS.write();
    senders.remove(filename);
}

/// Retry a failed/cancelled download by filename
pub fn retry_download(filename: &str) {
    // Reset status to Idle
    let mut queue = DOWNLOAD_QUEUE.write();
    if let Some(item) = queue.iter_mut().find(|i| i.filename.as_str() == filename) {
        item.status = DownloadStatus::Idle;
        item.downloaded_bytes = 0;
        item.current_rate = 0.0;
        item.est_time_remaining = None;
    }
    drop(queue);
    // Remove old task handle
    let mut tasks = DOWNLOAD_TASKS.write();
    tasks.remove(filename);
    // Restart downloads (read signals before spawn)
    let items = DOWNLOAD_QUEUE.read().clone();
    let dest_dir = config::get_download_dir();
    let max_concurrent = CONFIG.read().max_concurrent_downloads;
    let semaphore = Arc::new(Semaphore::new(max_concurrent as usize));
    spawn(async move {
        start_downloads_impl(items, dest_dir, semaphore).await;
    });
}

/// Open download directory using system opener
pub fn open_download_dir() {
    let dir = config::get_download_dir();
    if dir.exists() {
        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&dir)
                .spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&dir)
                .spawn();
        }
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg(&dir)
                .spawn();
        }
    }
}
