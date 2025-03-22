use dioxus::{logger::tracing::info, prelude::*};
use wasm_bindgen::prelude::*;
use web_sys::window;

use components_lib::text_editing::editor::{
    buffer::Buffer,
    cursor::CursorPosition,
    editor_view::EditorView,
    status_bar::StatusBar,
    theme::{Theme, available_themes},
    toolbar::Toolbar,
    file_dialog::NewFileDialog,
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
    let mut show_new_file_dialog = use_signal(|| false);
    // State to keep track of the currently open file handle
    let mut file_handle = use_signal(|| None::<web_sys::FileSystemFileHandle>);
    
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
        show_new_file_dialog.set(true);
    });

    let handle_create_file = use_callback(move |(new_filename, new_language): (String, String)| {
        buffer.set(Buffer::new());
        filename.set(Some(new_filename));
        language.set(Some(new_language));
        show_new_file_dialog.set(false);
    });

    let handle_cancel_new_file = use_callback(move |_: ()| {
        show_new_file_dialog.set(false);
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

    // Function to handle "Save As..." dialog
    let handle_save_as = use_callback(move |_: ()| {
        let window = web_sys::window().expect("no global `window` exists");
        let current_text = buffer.read().text();
        let current_filename = filename.read().clone().unwrap_or_else(|| "untitled.txt".to_string());

        // Use the File System Access API's showSaveFilePicker
        let js_save_as = format!(
            r#"
                (async function() {{
                    try {{
                        // Check if the File System Access API is supported
                        if (!('showSaveFilePicker' in window)) {{
                            throw new Error('File System Access API not supported');
                        }}

                        const options = {{
                            suggestedName: '{}',
                            types: [
                                {{
                                    description: 'Text Files',
                                    accept: {{ 'text/plain': ['.txt', '.rs', '.js', '.html', '.css', '.md', '.json', '.toml', '.yaml', '.yml'] }}
                                }}
                            ]
                        }};

                        const handle = await window.showSaveFilePicker(options);
                        const writable = await handle.createWritable();
                        await writable.write('{}');
                        await writable.close();

                        // Store file info
                        window._saveFileName = handle.name;
                        window._saveFileHandle = handle;

                        // Determine language from extension
                        const ext = handle.name.split('.').pop().toLowerCase();
                        let lang = 'plain';
                        
                        switch(ext) {{
                            case 'rs': lang = 'rust'; break;
                            case 'js': lang = 'javascript'; break;
                            case 'html': lang = 'html'; break;
                            case 'css': lang = 'css'; break;
                            case 'md': lang = 'markdown'; break;
                            case 'json': lang = 'json'; break;
                            case 'toml': lang = 'toml'; break;
                            case 'yaml':
                            case 'yml': lang = 'yaml'; break;
                        }}

                        // Return success with file info
                        return {{ success: true, name: handle.name, language: lang }};
                    }} catch (e) {{
                        console.error('Error in save as:", e);

                        // If File System Access API is not supported, fallback to download Blob
                        if (e.message === 'File System Access API not supported') {{
                            const blob = new Blob(['{}'], {{ type: 'text/plain' }});
                            const url = URL.createObjectURL(blob);
                            const anchor = document.createElement('a');
                            anchor.href = url;
                            anchor.download = '{}';
                            a.click();
                            URL.revokeObjectURL(url);
                            return {{ success: true, name: '{}', language: '{}', fallback: true }};
                        }}

                        return {{ success: false, error: e.toString() }};
                     }}
            }})()
            "#,
            current_filename,
            current_text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n"),
            current_text.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n"),
            current_filename,
            current_filename,
            language.read().clone().unwrap_or_else(|| "plain".to_string())
        );

        // Execute the JaveScript and handle the result
        let promise = js_sys::eval(&js_save_as);

        // Use a script to check results and call back to our Rust code
        let document = window.document().expect("should have a document on window");
        let script = document.create_element("script").expect("could not create script element");

        script.set_text_content(Some(&format!(
            r#"
                (async function() {{
                    try {{
                        const result = await {};
                        if (result && result.success) {{
                            // Update Rust state
                            window._updateFileInfo && window._updateFileInfo(result.name, result.language);

                            // Store file handle if not using fallback
                            if (!result.fallback) {{
                                window._saveFileHandle && window._storeFileHandle(window._savedFileHandle);
                            }}
                        }}
                    }} catch (e) {{
                        console.error("Error processing save result:", e); 
                    }}
                }})();
            "#,
            js_save_as
        )));

        document.body().expect("no body").append_child(&script).expect("could not append script");

        // Create callback functios for JavaScript to call back to Rust
        let update_name_lang = Closure::wrap(Box::new(move |name: String, lang: String| {
            filename.set(Some(name));
            language.set(Some(lang));
        }) as Box<dyn FnMut(String, String)>);

        // Store file handle
        let store_handle = Closure::wrap(Box::new(move |handle: web_sys::FileSystemFileHandle| {
            file_handle.set(Some(handle));
        }) as Box<dyn FnMut(web_sys::FileSystemFileHandle)>);

        // Attach callbacks to window
        let window_any = window.dyn_into::<js_sys::Object>().expect("could not cast window to Object");
        js_sys::Reflect::set(
            &window_any,
            &JsValue::from_str("_updateFileInfo"),
            &update_name_lang.as_ref()
        ).expect("Failed to set window._updateFileInfo");

        js_sys::Reflect::set(
            &window_any,
            &JsValue::from_str("_storeFileHandle"),
            &store_handle.as_ref()
        ).expect("Failed to set window._storeFileHandle");

        // Prevent the callbacks from being dropped
        update_name_lang.forget();
        store_handle.forget();
    });

    let handle_save_file = use_callback(move |_: ()| {
        // Check if we aleady have a file handle
        if let Some(handle) = file_handle() {
            // Get the current text
            let current_text = buffer.read().text();

            // Use JavaScript to save the existing file
            let js_save = 
                r#"
                (async function() {{
                    try {{
                        const handle = window._savedFileHandle;
                        if (!handle) {{
                            throw new Error("No file handle availabe to save the file");
                        }}

                        // Create a writable stream and write the current text
                        const writable = await arguments[0].createWritable();
                        await writable.write(arguments[0]);
                        await writable.close();
                        return true;
                    }} catch (e) {{
                        console.error('Error saving file:", e);
                        return false; 
                    }}
                }})()
            "#;

            // Execute the JavaScript with the current text as an argument
            let window = web_sys::window().expect("no global `window` exists");
            let js_function = js_sys::Function::new_with_args("text", js_save);
            let result = js_function.call1(
                &JsValue::NULL,
                &JsValue::from_str(&current_text)
            );

            if let Err(e) = result {
                // If there's an error, fall back to Save As...
                info!("Error saving file:{e:?}, falling back to Save As...");
                handle_save_as(());
            }
        } else {
            // No existing file handle, perform "Save As..."
            handle_save_as(());
        }
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
                on_save_as: handle_save_as,
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

            if show_new_file_dialog() {
                NewFileDialog {
                    theme: current_theme.clone(),
                    on_create: handle_create_file,
                    on_cancel: handle_cancel_new_file,
                }
            }
        }
    }
}