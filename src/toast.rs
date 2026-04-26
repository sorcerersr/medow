use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error(String),
}

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, PartialEq)]
struct ToastRenderData {
    id: u64,
    message: String,
    is_error: bool,
    class: String,
}

static TOAST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn next_id() -> u64 {
    TOAST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub static TOASTS: GlobalSignal<Vec<Toast>> = Signal::global(Vec::new);

pub fn show_toast(message: &str, toast_type: ToastType) {
    let id = next_id();
    let toast = Toast {
        id,
        message: message.to_string(),
        toast_type,
    };
    TOASTS.write().push(toast);
}

pub fn dismiss_toast(id: u64) {
    TOASTS.write().retain(|t| t.id != id);
}

#[component]
pub fn ToastContainer() -> Element {
    let toasts = TOASTS
        .read()
        .iter()
        .map(|t| ToastRenderData {
            id: t.id,
            message: t.message.clone(),
            is_error: matches!(&t.toast_type, ToastType::Error(_)),
            class: match &t.toast_type {
                ToastType::Success => "toast toast-success".to_string(),
                ToastType::Error(_) => "toast toast-error".to_string(),
            },
        })
        .collect::<Vec<_>>();

    let toast_data: Vec<(u64, String, bool, String)> = toasts
        .into_iter()
        .map(|t| (t.id, t.message, t.is_error, t.class))
        .collect();

    rsx! {
        div {
            class: "toast-container",
            for (id, msg, is_err, cls) in toast_data {
                div {
                    class: "{cls}",
                    span { class: "toast-message", "{msg}" }
                    if is_err {
                        button {
                            class: "toast-close",
                            onclick: move |_| dismiss_toast(id),
                            "×"
                        }
                    }
                }
            }
        }
    }
}
