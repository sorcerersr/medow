use crate::{
    pagination::{Pagination, SearchItem},
    APP_STATE, CONFIG, MEDOW_USER_AGENT,
};
use dioxus::prelude::*;
use mediathekviewweb::Mediathek;

/// Select the best video URL based on quality preference
fn select_best_url(item: &mediathekviewweb::models::Item, preferred: &str) -> (String, String) {
    let has_hd = item.url_video_hd.is_some();
    let has_sd = !item.url_video.is_empty();
    let has_lq = item.url_video_low.is_some();

    match preferred {
        "HD" if has_hd => (
            item.url_video_hd.clone().unwrap_or_default(),
            String::from("HD"),
        ),
        "HD" if has_sd => (item.url_video.clone(), String::from("SD")),
        "HD" if has_lq => (
            item.url_video_low.clone().unwrap_or_default(),
            String::from("LQ"),
        ),
        "SD" if has_sd => (item.url_video.clone(), String::from("SD")),
        "SD" if has_hd => (
            item.url_video_hd.clone().unwrap_or_default(),
            String::from("HD"),
        ),
        "SD" if has_lq => (
            item.url_video_low.clone().unwrap_or_default(),
            String::from("LQ"),
        ),
        "LQ" if has_lq => (
            item.url_video_low.clone().unwrap_or_default(),
            String::from("LQ"),
        ),
        "LQ" if has_sd => (item.url_video.clone(), String::from("SD")),
        "LQ" if has_hd => (
            item.url_video_hd.clone().unwrap_or_default(),
            String::from("HD"),
        ),
        _ => (String::from(""), String::from("")),
    }
}

pub async fn perform_search(mut pagination: Signal<Pagination>, query: String, offset: usize) {
    eprintln!("[search] query: {query}, offset: {offset}");
    APP_STATE.write().is_loading = true;
    let user_agent = match MEDOW_USER_AGENT.try_into() {
        Ok(ua) => ua,
        Err(error) => {
            eprintln!("[search] invalid user agent: {error}");
            APP_STATE.write().error = Some(format!("Invalid user agent: {error}"));
            APP_STATE.write().is_loading = false;
            return;
        }
    };
    let mediathek_client = match Mediathek::new(user_agent) {
        Ok(client) => client,
        Err(error) => {
            eprintln!("[search] failed to create client: {error}");
            APP_STATE.write().error = Some(format!("Failed to create client: {error}"));
            APP_STATE.write().is_loading = false;
            return;
        }
    };

    let query = mediathek_client
        .query(
            [
                mediathekviewweb::models::QueryField::Topic,
                mediathekviewweb::models::QueryField::Title,
            ],
            query,
        )
        .include_future(false)
        .sort_by(mediathekviewweb::models::SortField::Timestamp)
        .sort_order(mediathekviewweb::models::SortOrder::Descending)
        .size(15)
        .offset(offset);

    let search_result = match query.await {
        Ok(result) => {
            APP_STATE.write().is_loading = false;
            result
        }
        Err(error) => {
            eprintln!("[search] API error: {error}");
            APP_STATE.write().error = Some(format!("Search failed: {error}"));
            APP_STATE.write().is_loading = false;
            return;
        }
    };

    // Store total result count and current offset for pagination
    let mut pg = pagination.write();
    pg.total = search_result.query_info.total_results as usize;
    pg.offset = offset;
    drop(pg);
    // Get preferred quality from config
    let preferred_quality = CONFIG.read().quality_preference.clone();

    // Map SearchResult to SearchItem
    let mut search_items: Vec<SearchItem> = search_result
        .results
        .into_iter()
        .map(|item| {
            // Determine the best video URL based on quality preference
            let (video_url, quality) = select_best_url(&item, &preferred_quality);

            // Handle optional fields with defaults
            let timestamp = crate::utils::timestamp_to_german_datetime(item.timestamp);
            let duration = crate::utils::format_duration(&item.duration);

            SearchItem {
                selected: false,
                title: item.title,
                topic: item.topic,
                timestamp,
                duration,
                quality,
                video_url,
            }
        })
        .collect();

    // Update the items signal with the new search results
    pagination.write().items.clear();
    pagination.write().items.append(search_items.as_mut());
}
