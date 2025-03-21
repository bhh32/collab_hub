use dioxus::prelude::*;
use super::theme::Theme;

#[component]
pub fn Toolbar(
    theme: Theme,
    theme_names: Vec<String>,
    current_theme: String,
    on_theme_change: EventHandler<String>,
    on_new_file: EventHandler<()>,
    on_open_file: EventHandler<()>,
    on_save_file: EventHandler<()>,
) -> Element {
    let toolbar_style = format!(
        "display: flex; padding: 0.5rem; gap: 0.5rem;
         background-color: {}; color: {};",
         theme.ui.toolbar_bg, theme.ui.toolbar_fg
    );

    let button_style = format!(
        "padding: 0.25rem 0.5rem; border: none; border-radius: 4px;
         background-color: {}; color: {};",
         theme.ui.button, theme.ui.toolbar_fg
    );

    let select_style = format!(
        "padding: 0.25rem 0.5rem; border: none; border-radius: 4px;
         background-color: {}; color: {}; margin-left: auto;",
         theme.ui.button, theme.ui.toolbar_fg
    );

    rsx! {
        div {
            style: toolbar_style,
            button {
                style: button_style.clone(),
                onclick: move |_| on_new_file.call(()),
                "New"
            }
            button {
                style: button_style.clone(),
                onclick: move |_| on_open_file.call(()),
                "Open"
            }
            button {
                style: button_style.clone(),
                onclick: move |_| on_save_file.call(()),
                "Save"
            }

            select {
                style: select_style,
                value: current_theme.clone(),
                onchange: move |event| on_theme_change.call(event.value().clone()),
                    {
                        theme_names.iter().map(|theme_name| {
                        let name_value = theme_name.clone();
                        rsx! {
                            option {
                                value: name_value.clone(),
                                selected: name_value == current_theme,
                                "{name_value}"
                            }
                        }
                    })
                }
            }
        }
    }
}