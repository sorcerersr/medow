use std::collections::HashMap;

use dioxus::desktop::tao;
use dioxus::prelude::*;

mod config;
mod downloads_view;
mod pagination;
mod search_logic;
mod search_view;
mod settings_view;
mod toast;
mod utils;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.blue.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

const MEDOW_USER_AGENT: &str = "Mozilla/5.0 Linux Medow/0.1";

// Navigation function
pub(crate) fn navigate(view: View) {
    APP_STATE.write().view = view;
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
    // Selected items: video_url -> (title, topic), persists across pagination
    selected_items: HashMap<String, (String, String)>,
    // Generated download shell script
    download_script: Option<String>,
}

// init application wide state
static APP_STATE: GlobalSignal<AppState> = Signal::global(|| AppState {
    view: View::Search,
    error: Option::None,
    is_loading: false,
    selected_items: HashMap::new(),
    download_script: None,
});

// Global config signal
static CONFIG: GlobalSignal<config::AppConfig> = Signal::global(config::load_config);

fn main() {
    // There are some issues on wayland like the window buttons
    // not reacting - so fallback to x11
    std::env::set_var("GDK_BACKEND", "x11");

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
