use dioxus::prelude::*;

use crate::{config, navigate, View, APP_STATE, CONFIG};

#[component]
fn header_bar() -> Element {
    rsx! {
        header {
            class: "sticky-header",
            article {
                padding_bottom: 0,
                padding_top: 0,
                nav {
                    ul {
                        margin_left: "auto",
                        if APP_STATE.read().view != View::Search {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Search),
                                    "Search"
                                }
                            }
                        }
                        if APP_STATE.read().view != View::Download {
                            li {
                                button {
                                    class: "button",
                                    onclick: move |_| navigate(View::Download),
                                    "Downloads"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn settings_view() -> Element {
    let mut download_dir = use_signal(|| CONFIG.read().default_download_dir.clone());
    let mut quality = use_signal(|| CONFIG.read().quality_preference.clone());

    async fn save_settings(download_dir: &str, quality: &str) -> Result<(), String> {
        let app_config = config::AppConfig {
            default_download_dir: download_dir.to_string(),
            quality_preference: quality.to_string(),
        };
        config::save_config(&app_config)
    }

    rsx! {
        header_bar {}
        main {
            article {
                padding_bottom: 0,
                padding_top: 5,
                h1 { "Settings" }
                div {
                    class: "settings-group",
                    label { "Default Download Directory" }
                    div {
                        class: "dir-input-group",
                        input {
                            r#type: "text",
                            value: "{download_dir()}",
                            readonly: true,
                            class: "form-control",
                        }
                        button {
                            class: "button",
                            onclick: move |_| async move {
                                if let Some(selected) = rfd::FileDialog::new()
                                    .set_title("Select Download Directory")
                                    .pick_folder()
                                {
                                    let path = selected.to_string_lossy().to_string();
                                    download_dir.set(path.clone());
                                    if let Err(e) = save_settings(&path, &quality.read().clone()).await {
                                        crate::toast::show_toast(&format!("Save failed: {e}"), crate::toast::ToastType::Error(e));
                                    } else {
                                        crate::toast::show_toast("Settings saved", crate::toast::ToastType::Success);
                                        *CONFIG.write() = config::AppConfig {
                                            default_download_dir: path,
                                            quality_preference: quality.read().clone(),
                                        };
                                    }
                                }
                            },
                            "Browse"
                        }
                    }
                    label { "Quality Preference" }
                    select {
                        class: "form-control",
                        value: "{quality()}",
                        onchange: move |e| async move {
                            let new_quality = e.value();
                            quality.set(new_quality.clone());
                            if let Err(e) = save_settings(&download_dir.read().clone(), &new_quality).await {
                                crate::toast::show_toast(&format!("Save failed: {e}"), crate::toast::ToastType::Error(e));
                            } else {
                                crate::toast::show_toast("Settings saved", crate::toast::ToastType::Success);
                                *CONFIG.write() = config::AppConfig {
                                    default_download_dir: download_dir.read().clone(),
                                    quality_preference: new_quality,
                                };
                            }
                        },
                        option { value: "HD", "HD (High Quality)" }
                        option { value: "SD", "SD (Standard Quality)" }
                        option { value: "LQ", "LQ (Low Quality)" }
                    }
                }
            }
        }
    }
}
