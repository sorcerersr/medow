use dioxus::desktop::tao;
use dioxus::prelude::*;

mod config;
mod downloads;
mod downloads_view;
mod pagination;
mod search_logic;
mod search_view;
mod settings_view;
mod state;
mod toast;
mod utils;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.blue.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

const MEDOW_USER_AGENT: &str = "Mozilla/5.0 Linux Medow/0.1";

// Navigation function
pub(crate) fn navigate(view: View) {
    let mut app_state = APP_STATE.write();
    app_state.view = view;
    drop(app_state);
    // Save state on navigation
    save_app_state();
}

// Enumeration to define the navigatable views
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum View {
    Search,
    Settings,
    Download,
}

// struct representing a global application wide state
struct AppState {
    view: View,
    error: Option<String>,
    is_loading: bool,
}

// init application wide state
static APP_STATE: GlobalSignal<AppState> = Signal::global(|| AppState {
    view: View::Search,
    error: Option::None,
    is_loading: false,
});

// Global config signal
static CONFIG: GlobalSignal<config::AppConfig> = Signal::global(config::load_config);

// Saved state for app initialization (set before launch, read inside runtime)
static SAVED_STATE: std::sync::Mutex<Option<state::AppState>> = std::sync::Mutex::new(None);

// Save application state to disk
fn save_app_state() {
    let current_view = APP_STATE.read().view;
    let view_str = match current_view {
        View::Search => "Search",
        View::Download => "Download",
        View::Settings => "Settings",
    };

    // Get download queue
    let download_queue = downloads::get_selected_items();

    let app_state = state::AppState {
        current_view: view_str.to_string(),
        search_term: String::new(),
        pagination: None,
        download_queue,
    };

    if let Err(e) = state::save_state(&app_state) {
        eprintln!("Failed to save state: {e}");
    }
}

fn main() {
    // There are some issues on wayland like the window buttons
    // not reacting - so fallback to x11
    std::env::set_var("GDK_BACKEND", "x11");

    // Load saved state (pure file I/O, no Dioxus runtime needed)
    let saved_state = state::load_state();

    // Store in static for App component to read inside runtime
    *SAVED_STATE.lock().unwrap() = Some(saved_state);

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new().with_menu(None).with_window(
                tao::window::WindowBuilder::new()
                    .with_title("Medow")
                    .with_maximized(true),
            ),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    // Restore state inside Dioxus runtime
    if let Some(saved_state) = SAVED_STATE.lock().unwrap().take() {
        if !saved_state.download_queue.is_empty() {
            *downloads::DOWNLOAD_QUEUE.write() = saved_state.download_queue;
        }

        let initial_view = match saved_state.current_view.as_str() {
            "Download" => View::Download,
            "Settings" => View::Settings,
            _ => View::Search,
        };
        APP_STATE.write().view = initial_view;
    }

    let view = APP_STATE.read().view;
    let current_view = match view {
        View::Search => rsx! { search_view::search_view {} },
        View::Download => rsx! { downloads_view::downloads_view {} },
        View::Settings => rsx! { settings_view::settings_view {} },
    };

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "layout-container", {current_view} }
        toast::ToastContainer {}
    }
}
