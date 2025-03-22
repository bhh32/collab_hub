use dioxus::prelude::*;
use super::theme::Theme;

#[component]
pub fn NewFileDialog(
    theme: Theme,
    on_create: EventHandler<(String, String)>, // (filename, language)
    on_cancel: EventHandler<()>, // No parameters for cancel
) -> Element {
    let mut filename = use_signal(|| String::from("untitled.rs"));
    let mut language = use_signal(|| String::from("rust"));

    let dialog_style = format!(
        "position: fixed; top: 0; left: 0; right: 0; bottom: 0;
         background-color: rgba(0, 0, 0, 0.7);
         display: flex; align-items: center; justify-content: center;
         z-index: 100;",
    );

    let panel_style = format!(
        "background-color: {}; color: {}; padding: 1.5rem;
         border-radius: 4px; width: 400px;",
         theme.background, theme.foreground
    );

    let input_style = format!(
        "width: 100%; padding: 0.5rem; margin: 0.5rem 0;
         background-color: {}; color: {}; border: 1px solid #555;
         border-radius: 3px;",
         theme.background, theme.foreground
    );

    let select_style = format!(
        "width: 100%; padding: 0.5rem; margin: 0.5rem 0;
         background-color: {}, color: {}; border: 1px solid #555;
         border-radius: 3px;",
         theme.background, theme.foreground
    );

    let button_style = format!(
        "padding: 0.5rem 1rem; margin-left: 0.5rem;
         border: none; border-radius: 3px; cursor: pointer;"
    );

    let primary_button_style = format!(
        "{} background-color: #0078d7; color: white;",
        button_style
    );

    let secondary_button_style = format!(
        "{} background-color: #333; color: white;",
        button_style
    );

    let handle_submit = move |_| {
        on_create.call((filename(), language()));
    };

    let handle_cancel = move |_| {
        on_cancel.call(());
    };

    // Auto-update the file extension based on the language selection
    let mut update_extension = move |selected_lang: String| {
        language.set(selected_lang.clone());

        // Extract the base name without extension
        let filename_value = filename();
        let base_name = if let Some(dot_pos) = filename_value.rfind('.') {
            &filename_value[0..dot_pos]
        } else {
            &filename_value
        };

        // Set new extension based on language
        let extension = match selected_lang.as_str() {
            "rust" => "rs",
            "javascript" => "js",
            "html" => "html",
            "css" => "css",
            "markdown" => "md",
            "json" => "json",
            "toml" => "toml",
            "yaml" => "yaml",
            _ => "txt", // default to plain text
        };

        filename.set(format!("{}.{}", base_name, extension));
    };

    rsx! {
        div {
            style: dialog_style,
            div {
                style: panel_style,
                h3 { "Create New File" }

                div {
                    style: "margin-bottom: 1rem;",
                    label {
                        r#for: "filename-input",
                        "Filename:"
                    }
                    input {
                        id: "filename-input",
                        style: input_style,
                        value: filename(),
                        oninput: move |e| filename.set(e.value().clone()),
                    }
                }

                div {
                    style: "margin-bottom: 1.5rem;",
                    label {
                        r#for: "language-select",
                        "Language:"
                    }
                    select {
                        id: "language-select",
                        style: select_style,
                        value: language(),
                        onchange: move |e| update_extension(e.value().clone()),

                        option { value: "rust", "Rust" }
                        option { value: "javascript", "JavaScript" }
                        option { value: "html", "HTML" }
                        option { value: "css", "CSS" }
                        option { value: "markdown", "Markdown" }
                        option { value: "json", "JSON" }
                        option { value: "toml", "TOML" }
                        option { value: "yaml", "YAML" }
                        option { value: "plain", "Plain Text" }
                    }
                }

                div {
                    style: "display: flex; justify-content: flex-end;",
                    button {
                        style: secondary_button_style,
                        onclick: handle_cancel,
                        "Cancel"
                    }
                    button {
                        style: primary_button_style,
                        onclick: handle_submit,
                        "Create"
                    }
                }
            }
        }
    }
}