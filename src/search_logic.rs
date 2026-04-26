use crate::{
    pagination::{Pagination, SearchItem},
    APP_STATE, MEDOW_USER_AGENT,
};
use dioxus::prelude::*;
use mediathekviewweb::Mediathek;

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

    // Store total result count from API for pagination
    pagination.write().total = search_result.query_info.total_results as usize;

    // Map SearchResult to SearchItem
    let mut search_items: Vec<SearchItem> = search_result
        .results
        .into_iter()
        .map(|item| {
            // Determine the best video URL available
            let (video_url, quality) = if !item.url_video.is_empty() {
                (item.url_video, String::from("SD"))
            } else if let Some(url) = item.url_video_hd {
                (url, String::from("HD"))
            } else {
                (
                    item.url_video_low.unwrap_or(String::from("")),
                    String::from("LQ"),
                )
            };

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
