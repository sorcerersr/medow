use std::fs;
use std::path::PathBuf;

use dirs::config_dir;
use serde::{Deserialize, Serialize};

use crate::downloads::DownloadItem;
use crate::pagination::Pagination;

pub const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppState {
    pub current_view: String,
    pub search_term: String,
    pub pagination: Option<PaginationState>,
    pub download_queue: Vec<DownloadItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginationState {
    pub total: usize,
    pub offset: usize,
    pub items: Vec<crate::pagination::SearchItem>,
}

impl From<Pagination> for PaginationState {
    fn from(pg: Pagination) -> Self {
        PaginationState {
            total: pg.total,
            offset: pg.offset,
            items: pg.items,
        }
    }
}

impl From<PaginationState> for Pagination {
    fn from(state: PaginationState) -> Self {
        Pagination {
            total: state.total,
            offset: state.offset,
            items: state.items,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_view: String::from("Search"),
            search_term: String::new(),
            pagination: None,
            download_queue: Vec::new(),
        }
    }
}

fn state_path() -> PathBuf {
    let dir = config_dir().map(|p| p.join("medow")).unwrap_or_else(|| {
        if let Some(home) = dirs::home_dir() {
            home.join(".config").join("medow")
        } else {
            PathBuf::from("medow")
        }
    });
    dir.join(STATE_FILE)
}

/// Load application state from disk
pub fn load_state() -> AppState {
    let path = state_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(state) => {
                    println!("State loaded from {:?}", path);
                    state
                }
                Err(e) => {
                    eprintln!("Failed to parse state: {e}, using defaults");
                    AppState::default()
                }
            },
            Err(e) => {
                eprintln!("Failed to read state: {e}, using defaults");
                AppState::default()
            }
        }
    } else {
        println!("No state file found, using defaults");
        AppState::default()
    }
}

/// Save application state to disk
pub fn save_state(state: &AppState) -> Result<(), String> {
    let path = state_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create state dir: {e}"))?;
    }

    // Serialize to JSON
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {e}"))?;

    // Write to file
    fs::write(&path, content).map_err(|e| format!("Failed to write state: {e}"))?;

    println!("State saved to {:?}", path);
    Ok(())
}
