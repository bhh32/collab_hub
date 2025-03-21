use dioxus::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use super::{buffer::Buffer, cursor::CursorPosition, theme::Theme, highlighter::SyntaxHighlighter};

#[component]
pub fn EditorView(
    buffer: Buffer,
    theme: Theme,
    on_buffer_change: EventHandler<Buffer>,
    on_cursor_move: EventHandler<CursorPosition>,
    language: Option<String>,
) -> Element {
    let mut textarea = use_signal(|| None::<HtmlTextAreaElement>);
    let mut cursor = use_signal(|| CursorPosition::default());

    let style = format!(
        "position: absolute; top: 0; left: 0; right: 0; bottom: 0; padding: 0.5rem;
         resize: none; outline: none; border: none;
         background-color: transparent; color: transparent; caret-color: {};
         font-family: 'Fira Code', monospace; font-size: 14px; line-height: 1.5;
         white-space: pre; tab-size: 4; z-index: 2;",
        theme.cursor
    );

    // Create a syntax highlighter for the specified language
    let lang = language.clone().unwrap_or_else(|| "plain".to_string());
    let highlighter = SyntaxHighlighter::new(lang, theme.clone());

    // Generate highlighted HTML
    let highlighted_code = highlighter.highlight(&buffer.text());

    // Handle keyboard events including tab
    let buffer_tab_event = buffer.clone();
    let handle_keydown = use_callback(move |event: Event<KeyboardData>| {
        // Check if it's the Tab key
        if event.key() == Key::Tab {
            // We can't prevent default here directly, but we'll handle it specially
            
            if let Some(textarea_ele) = textarea() {
                if let Ok(Some(start)) = textarea_ele.selection_start() {
                    let current_offset = start as usize;
                    
                    // Create a new buffer with the tab (4 spaces)
                    let mut new_buffer = buffer_tab_event.clone();
                    let _ = new_buffer.insert(current_offset, "    "); // 4 spaces for tab
                    on_buffer_change.call(new_buffer);

                    // Update the cursor position
                    let new_position = CursorPosition {
                        offset: current_offset + 4,
                        line: cursor.with(|c| c.line),
                        column: cursor.with(|c| c.column) + 4,
                    };

                    cursor.set(new_position);
                    on_cursor_move.call(new_position);

                    // Need to update the textarea's selection position manually
                    let _ = textarea_ele.set_selection_range(
                        (current_offset + 4) as u32,
                        (current_offset + 4) as u32,
                    );
                }
            }
        }
    });

    let buffer_input = buffer.clone();
    let mut handle_input = use_callback(move |event: Event<FormData>| {
        let new_text = event.value().clone();
        let buffer_text = buffer_input.text();

        if new_text != buffer_text {
            let new_buffer = Buffer::from_str(&new_text, buffer_input.clone().filename().cloned());
            on_buffer_change.call(new_buffer);
        }
    });

    // This function will do all the work for updating cursor position
    // but doesn't take any parameters - we'll call it from the event handlers
    let mut update_cursor = move || {
        if let Some(textarea_elem) = textarea() {
            if let Ok(Some(position)) = textarea_elem.selection_start() {
                let selection_start = position as usize;

                // Calculate line and column
                let text = textarea_elem.value();
                let line = text[..selection_start].matches('\n').count();
                let last_newline = text[..selection_start].rfind('\n').map(|line_num| line_num + 1).unwrap_or(0);
                let column = selection_start - last_newline;

                let new_position = CursorPosition {
                    offset: selection_start,
                    line,
                    column,
                };

                if cursor() != new_position {
                    cursor.set(new_position);
                    on_cursor_move.call(new_position);
                }
            }
        }
    };

    // Split the callbacks to handle different event types
    let handle_keyup = use_callback(move |_: Event<KeyboardData>| {
        update_cursor();
    });

    let handle_selection_change = use_callback(move |_: Event<SelectionData>| {
        update_cursor();
    });

    // Set up the textarea and event handlers
    let setup_textarea = move |_| {
        // Set the textarea reference
        let element = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("editor-textarea"))
            .and_then(|ele| ele.dyn_into::<HtmlTextAreaElement>().ok());

        if let Some(textarea_ele) = element {
            textarea.set(Some(textarea_ele.clone()));
            
            // Add a keydown event listener to prevent default tab behavior
            let tab_handler = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if event.key() == "Tab" {
                    event.prevent_default();
                    // The keydown handler in Dioxus will handle the rest
                }
            }) as Box<dyn FnMut(_)>);
            
            let _ = textarea_ele.add_event_listener_with_callback(
                "keydown",
                tab_handler.as_ref().unchecked_ref(),
            );

            let window = web_sys::window().expect("no global window");
            let document = window.document().expect("no document");

            let script = document.create_element("script").expect("couldn't create script");
            script.set_text_content(Some(r#"
                (function() {
                    const textarea = document.getElementById('editor-textarea');
                    const highlightLayer = document.getElementById('highlight-layer');

                    if (textarea && highlightLayer) {
                        textarea.addEventListener('scroll', function() {
                            highlightLayer.scrollTop = textarea.scrollTop;
                            highlightLayer.scrollLeft = textarea.scrollLeft;
                        });
                    }
                })();
            "#));

            document.body().expect("no body").append_child(&script).expect("couldn't append scroll sync script");
            
            // Prevent tab_handler from being dropped
            tab_handler.forget();
        }
    };

    // Sync the scrolling
    let sync_scroll = move |_| {
        if let Some(textarea_ele) = textarea() {
            let window = web_sys::window().expect("no window");
            let document = window.document().expect("no document");

            if let Some(highlight_div) = document.get_element_by_id("highlight-layer") {
                highlight_div.set_scroll_top(textarea_ele.scroll_top());
                highlight_div.set_scroll_left(textarea_ele.scroll_left());
            }
        }
    };

    rsx! {
        div {
            style: "height: 100%; position: relative;",

            // Add a div for the syntax highlighted text
            div {
                id: "highlight-layer",
                style: format!(
                    "position: absolute; top: 0; left: 0; right: 0; bottom: 0; padding: 0.5rem;
                     pointer-events: none; overflow: auto; white-space: pre;
                     font-family: 'Fira Code', monospace; font-size: 14px; line-height: 1.5;
                     tab-size: 4; z-index: 1; background-color: {}; color: {};",
                     theme.background, theme.foreground
                ),
                dangerous_inner_html: format!("{highlighted_code}"),
            }
            
            textarea {
                id: "editor-textarea",
                value: buffer.text(),
                style: style,
                spellcheck: false,
                onmounted: setup_textarea,
                onkeydown: handle_keydown,
                oninput: handle_input,
                onselectionchange: handle_selection_change,
                onkeyup: handle_keyup,
                onscroll: sync_scroll,
            }
        }
    }
}