pub mod editor_view;
pub mod buffer;
pub mod cursor;
pub mod theme;
pub mod status_bar;
pub mod toolbar;
pub mod highlighter;

use dioxus::prelude::*;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};
use web_sys::{
    window,
    FileReader,
    Event as WebEvent,
    HtmlInputElement,
};
use js_sys::{Uint8Array, ArrayBuffer};

use buffer::Buffer;
use cursor::CursorPosition;
use editor_view::EditorView;
use status_bar::StatusBar;
use toolbar::Toolbar;
use theme::{Theme, available_themes};

#[component]
pub fn CodeEditor() -> Element {
    // Application State
    let mut buffer = use_signal(|| Buffer::new());
    let mut cursor_position = use_signal(|| CursorPosition::default());
    let mut filename = use_signal(|| None::<String>);
    let mut language = use_signal(|| Some("plaintext".to_string()));

    // Theme State
    let themes = available_themes();
    let theme_names = themes.iter().map(|theme| theme.name.clone()).collect::<Vec<_>>();
    let mut current_theme_idx = use_signal(|| 0);

    // Event Handlers
    let handle_buffer_change = move |new_buffer: Buffer| {
        buffer.set(new_buffer);
    };

    let handle_cursor_move = move |new_cursor: CursorPosition| {
        cursor_position.set(new_cursor);
    };

    let themes_handler_clone = themes.clone();
    let handle_theme_change = move |theme_name: String| {
        if let Some(idx) = themes_handler_clone.iter().position(|theme| theme.name == theme_name) {
            current_theme_idx.set(idx);
        }
    };

    let handle_new_file = move |_| {
        buffer.set(Buffer::new());
        filename.set(None);
        language.set(Some("plaintext".to_string()));
    };

    let handle_open_file = move |_| {
        // Creat file input element
        let window = window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");

        let input: HtmlInputElement = document
            .create_element("input")
            .expect("could not create input")
            .dyn_into::<HtmlInputElement>()
            .expect("not an input element");

        input.set_type("file");
        input.set_accept(".rs,.txt,.md,.json,.js,.toml,.yaml,.yml,.html,.css");
        let input_clone = input.clone();

        // Create closures for the onchange event
        let mut buffer_setter = buffer.clone();
        let mut filename_setter = filename.clone();
        let mut language_setter = language.clone();

        let onchange = Closure::wrap(Box::new(move |_: WebEvent| {
            if let Some(file) = input_clone.files().and_then(|files| files.get(0)) {
                let file_name = file.name();
                let _file_name_str: &str = &file_name;

                // Detect language from file extension
                let lang_string = if file_name.ends_with(".rs") {
                    "rust"
                } else if file_name.ends_with(".js") {
                    "javascript"
                } else if file_name.ends_with(".html") {
                    "html"
                } else if file_name.ends_with(".css") {
                    "css"
                } else if file_name.ends_with(".md") {
                    "markdown"
                } else if file_name.ends_with(".json") {
                    "json"
                } else if file_name.ends_with(".toml") {
                    "toml"
                } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                    "yaml"
                } else {
                    "plaintext"
                };

                // Create file reader
                let reader = FileReader::new().expect("could not create file reader");

                let reader_clone = reader.clone();
                let onload = Closure::wrap(Box::new(move |_: WebEvent| {
                    if let Ok(ab) = reader_clone.result() {
                        let array_buffer: ArrayBuffer = ab.dyn_into().expect("not an array buffer");
                        let uint8_array = Uint8Array::new(&array_buffer);
                        let mut buf = vec![0; uint8_array.length() as usize];
                        uint8_array.copy_to(&mut buf);

                        if let Ok(text) = String::from_utf8(buf) {
                            buffer_setter.set(Buffer::from_str(&text, Some(file_name.to_string().clone())));
                            filename_setter.set(Some(file_name.to_string().clone()));
                            language_setter.set(Some(lang_string.to_string()));
                        }
                    }
                }) as Box<dyn FnMut(_)>);
                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget(); // Prevent memory leaks

                reader.read_as_array_buffer(&file).expect("failed to read file");
            }
        }) as Box<dyn FnMut(_)>);

        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();

        input.click();
    };

    let handle_save_file = move |_| {
        let current_text = buffer.read().text();
        let current_filename = filename.read().clone().unwrap_or_else(|| "untitled.txt".to_string());

        // Create a download link
        let window = window().expect("no global window exists");
        let document = window.document().expect("should have a document on window");

        let blob_parts = js_sys::Array::new();
        blob_parts.push(&JsValue::from_str(&current_text));

        let mut options = web_sys::BlobPropertyBag::new();
        options.type_("text/plain");

        let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options)
            .expect("could not create blob");

        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .expect("could not create object URL");

        let anchor: web_sys::HtmlAnchorElement = document
            .create_element("a")
            .expect("could not create anchor")
            .dyn_into()
            .expect("not an anchor element");

        anchor.set_href(&url);
        anchor.set_download(&current_filename);
        anchor.click(); // Trigger the download

        web_sys::Url::revoke_object_url(&url).expect("could not revoke object URL");
    };

    // Get current theme
    let current_theme = &themes[current_theme_idx()];
    let current_theme_name = current_theme.name.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; overflow: hidden;",
            Toolbar {
                theme: current_theme.clone(),
                theme_names: theme_names,
                current_theme: current_theme_name,
                on_theme_change: handle_theme_change,
                on_new_file: handle_new_file,
                on_open_file: handle_open_file,
                on_save_file: handle_save_file,
            }

            div {
                style: "flex: 1; overflow: hidden;",
                EditorView {
                    buffer: buffer(),
                    theme: current_theme.clone(),
                    on_buffer_change: handle_buffer_change,
                    on_cursor_move: handle_cursor_move,
                }
            }

            StatusBar {
                theme: current_theme.clone(),
                filename: filename(),
                language: language(),
                cursor_line: cursor_position().line,
                cursor_column: cursor_position().column,
                total_lines: buffer().line_count(),
            }
        }
    }
}