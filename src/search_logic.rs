use dioxus::prelude::*;
use mediathekviewweb::Mediathek;
use crate::{
    pagination::{Pagination, SearchItem},
    APP_STATE, MEDOW_USER_AGENT,
};

pub async fn perform_search(mut pagination: Signal<Pagination>, query: String, offset: usize) {
    println!("in the search callback with query string {query}");
    let mediathek_client = match Mediathek::new(MEDOW_USER_AGENT.try_into().unwrap()) {
        Ok(client) => client,
        Err(error) => {
            APP_STATE.write().error = Some(format!("{error:?}"));
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
        Ok(result) => result,
        Err(error) => {
            APP_STATE.write().error = Some(format!("{error:?}"));
            return;
        }
    };

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
                timestamp: timestamp,
                duration: duration,
                quality: quality,
                video_url,
            }
        })
        .collect();

    // Update the items signal with the new search results
    pagination.write().items.clear();
    pagination.write().items.append(search_items.as_mut());
}
