use dioxus::desktop::tao;
use dioxus::prelude::*;

mod pagination;
mod search_logic;
mod search_view;
mod utils;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const PICO_CSS: Asset = asset!("/assets/pico.blue.min.css");
const MAIN_CSS: Asset = asset!("/assets/main.css");

const MEDOW_USER_AGENT: &str = "Mozilla/5.0 Linux Medow/0.1";

// Enumeration to define the navigatable views
#[derive(Clone, Copy)]
enum View {
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

fn main() {
    // There are some issues on wayland like the window buttons
    // not reacting - so fallback to x11
    std::env::set_var("GDK_BACKEND", "x11");

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_menu(None)
                .with_window(tao::window::WindowBuilder::new().with_maximized(true)),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: PICO_CSS }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div { class: "layout-container", search_view::search_view {} }

    }
}
