use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::window;

use components_lib::text_editing::editor::{
    buffer::Buffer,
    cursor::CursorPosition,
    editor_view::EditorView,
    status_bar::StatusBar,
    theme::{Theme, available_themes},
    toolbar::Toolbar,
};

fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Launch the web app
    launch(App);
}

#[component]
fn App() -> Element {
    // Application State
    let mut buffer = use_signal(|| Buffer::new());
    let mut cursor_position = use_signal(|| CursorPosition::default());
    let mut filename = use_signal(|| None::<String>);
    let mut language = use_signal(|| Some("plain".to_string()));

    // Theme State
    let themes = available_themes();
    let themes_clone = themes.clone();
    let theme_names = themes.iter().map(|theme| theme.name.clone()).collect::<Vec<_>>();
    let mut current_theme_idx = use_signal(|| 0);

    // Event handlers
    let handle_buffer_change = use_callback(move |new_buffer: Buffer| {
        buffer.set(new_buffer);
    });

    let handle_cursor_move = use_callback(move |position: CursorPosition| {
        cursor_position.set(position);
    });

    let handle_theme_change = use_callback(move |theme_name: String| {
        if let Some(idx) = themes_clone.iter().position(|theme| theme.name == theme_name) {
            current_theme_idx.set(idx);
        }
    });

    let handle_new_file = use_callback(move |_| {
        buffer.set(Buffer::new());
        filename.set(None);
        language.set(Some("plain".to_string()));
    });

    let handle_open_file = use_callback(move |_| {
        // Create file input element
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");

        let input_element = document
            .create_element("input")
            .expect("could not create input element")
            .dyn_into::<web_sys::HtmlInputElement>()
            .expect("could not cast to HtmlInputElement");

        input_element.set_type("file");
        input_element.set_accept(".rs, .txt, .md, .json, .toml, .yaml, .yml, .js, .html, .css, .c, .cpp, .h, .py");

        // Create closures for the onchange event
        let buffer_setter = buffer.clone();
        let filename_setter = filename.clone();
        let language_setter = language.clone();

        let onchange = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().expect("event has no target");
            let input = target.dyn_into::<web_sys::HtmlInputElement>().expect("not an input element");

            if let Some(file_list) = input.files() {
                if let Some(file) = file_list.get(0) {
                    let file_name = file.name();

                    let lang = if file_name.ends_with(".rs") {
                        "rust"
                    } else if file_name.ends_with(".js") {
                        "javascript"
                    } else if file_name.ends_with(".html") {
                        "html"
                    } else if file_name.ends_with(".css") {
                        "css"
                    } else if file_name.ends_with("md") {
                        "markdown"
                    } else if file_name.ends_with(".json") {
                        "json"
                    } else if file_name.ends_with(".toml") {
                        "toml"
                    } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                        "yaml"
                    } else if file_name.ends_with(".py") {
                        "python"
                    } else {
                        "plain"
                    };

                    // Create a file reader
                    let reader = web_sys::FileReader::new().expect("could not create file reader");
                    let reader_clone = reader.clone();

                    let mut buf_setter = buffer_setter.clone();
                    let mut name_setter = filename_setter.clone();
                    let mut lang_setter = language_setter.clone();

                    let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        if let Ok(result) = reader.result() {
                            let array_buffer = result.dyn_into::<js_sys::ArrayBuffer>().expect("not an ArrayBuffer");
                            let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                            let mut buf = vec![0; uint8_array.length() as usize];
                            uint8_array.copy_to(&mut buf);

                            if let Ok(text) = String::from_utf8(buf) {
                                buf_setter.set(Buffer::from_str(&text, Some(file_name.clone())));
                                name_setter.set(Some(file_name.clone()));
                                lang_setter.set(Some(lang.to_string()));
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    reader_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
                    onload.forget(); // Prevent the closure from being dropped

                    let _ = reader_clone.read_as_array_buffer(&file);
                }
            }
        }) as Box<dyn FnMut(_)>);

        input_element.set_onchange(Some(onchange.as_ref().unchecked_ref()));
        onchange.forget();

        input_element.click(); // Trigger the file input dialog
    });

    let handle_save_file = use_callback(move |_: ()| {
        let current_text = buffer.read().text();
        let current_filename = filename.read().clone().unwrap_or_else(|| "untitled.txt".to_string());

        // Create a download link
        let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");

        let blob_parts = js_sys::Array::new();
        blob_parts.push(&JsValue::from_str(&current_text));

        let mut options = web_sys::BlobPropertyBag::new();
        options.type_("text/plain");

        let blob = web_sys::Blob::new_with_str_sequence_and_options(&blob_parts, &options)
            .expect("could not create blob");

        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .expect("could not create object URL");

        let anchor = document
            .create_element("a")
            .expect("could not create anchor")
            .dyn_into::<web_sys::HtmlAnchorElement>()
            .expect("not an anchor element");

        anchor.set_href(&url);
        anchor.set_download(&current_filename);

        // Append, click and remove to trigger download
        document.body().expect("document has no body").append_child(&anchor).expect("could not append anchor");
        anchor.click(); // Trigger the download
        document.body().expect("document has no body").remove_child(&anchor).expect("could not remove anchor");

        web_sys::Url::revoke_object_url(&url).expect("could not revoke object URL");
    });

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