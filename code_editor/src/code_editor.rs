use dioxus::prelude::*;
use components_lib::editor::{
    editor_core::{
        Buffer,
        CursorPosition,
    },
    dialogs::file_dialog::NewFileDialog,
    panels::{
        StatusBar,
        menus::{
            menu_config::get_default_editor_menus,
            menu::{
                MenuBar,
                MenuHandler,
            }
        }
    }
};
use components_lib::available_themes;
use crate::code_editor_view::EditorView;
use wasm_bindgen::{
    prelude::*,
    JsCast,
};



/// Handles the Editor menu components State and Events
#[derive(Clone, PartialEq)]
pub struct EditorMenuHandler {
    // State fields for menu operations
    pub buffer_has_changes: bool,
    pub has_filename: bool,
    pub theme_is_light: Signal<bool>, 
    pub theme_is_dark: Signal<bool>,
}

impl Default for EditorMenuHandler {
    fn default() -> Self {
        Self {
            buffer_has_changes: false,
            has_filename: false,
            theme_is_light: Signal::new(false),
            theme_is_dark: Signal::new(true),
        }
    }
}

impl EditorMenuHandler {
    pub fn new(
        buffer_has_changes: bool,
        has_filename: bool,
        theme_is_light: bool,
    ) -> Self {
        let mut new_handler = Self {
            buffer_has_changes,
            has_filename,
            ..Default::default()
        };

        // Set the theme signals based on the passed values
        if theme_is_light {
            new_handler.theme_is_light.set(true);
            new_handler.theme_is_dark.set(false);
        } else {
            new_handler.theme_is_dark.set(true);
            new_handler.theme_is_light.set(false);
        }

        new_handler
    }
}

impl MenuHandler for EditorMenuHandler {
    fn handle_menu_action(&mut self, action_id: &str) {
        // Just pass the action directly to a global handler
        // In WASM, we can use JavaScript to trigger the actions
        match action_id {
            "file.new" => {
                let _ = js_sys::eval("window._editorActions && window._editorActions.newFile()");
            },
            "file.open" => {
                let _ = js_sys::eval("window._editorActions && window._editorActions.openFile()");
            },
            "file.save" => {
                let _ = js_sys::eval("window._editorActions && window._editorActions.saveFile()");
            },
            "file.save_as" => {
                let _ = js_sys::eval("window._editorActions && window._editorActions.saveFileAs()");
            },
            "file.exit" => {
                let _ = js_sys::eval("window.close();");
            },
            "edit.cut" => {
                let _ = js_sys::eval("document.execCommand('cut');");
            },
            "edit.copy" => {
                let _ = js_sys::eval("document.execCommand('copy');");
            },
            "edit.paste" => {
                let _ = js_sys::eval("document.execCommand('paste');");
            },
            "view.theme.light" => {
                if !*self.theme_is_light.read() {
                    self.theme_is_light.set(true);
                    self.theme_is_dark.set(false);

                    let _ = js_sys::eval("window._editorActions && window._editorActions.setTheme('light')");
                }
            },
            "view.theme.dark" => {
                if !*self.theme_is_dark.read() {
                    self.theme_is_dark.set(true);
                    self.theme_is_light.set(false);

                    let _ = js_sys::eval("window._editorActions && window._editorActions.setTheme('dark')");
                }
            },
            "help.about" => {
                let _ = js_sys::eval(
                    "alert('Collab Hub - Code Editor\\nA lightweight code editor built with Rust, Dioxus, and WebAssembly.');"
                );
            },
            _ => {}
        }
    }

    fn is_item_enabled(&self, item_id: &str) -> bool {
        match item_id {
            // Disable Save if nothing has changed or no file is open
            "file.save" => self.buffer_has_changes && self.has_filename,
            _ => true,
        }
    }

    fn is_item_checked(&self, item_id: &str) -> Option<bool> {
        match item_id {
            "view.theme.light" => Some(*self.theme_is_light.read()),
            "view.theme.dark" => Some(*self.theme_is_dark.read()),
            _ => None,
        }
    }
}

#[component]
pub fn CodeEditor() -> Element {
    // Application State
    let mut buffer = use_signal(|| Buffer::new());
    let mut cursor_position = use_signal(|| CursorPosition::default());
    let mut filename = use_signal(|| None::<String>);
    let mut language = use_signal(|| Some("plaintext".to_string()));
    let mut file_handle = use_signal(|| None::<web_sys::FileSystemFileHandle>);
    let mut show_new_file_dialog = use_signal(|| false);
    let menu_items = get_default_editor_menus();

    // Theme State
    let themes = available_themes();
    let current_theme_idx = use_signal(|| 0);

    // Event Handlers
    let handle_buffer_change = move |new_buffer: Buffer| {
        buffer.set(new_buffer);
    };

    let handle_cursor_move = move |new_cursor: CursorPosition| {
        cursor_position.set(new_cursor);
    };

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

    let handle_open_file = use_callback(move |_: ()| {
        let window = web_sys::window().expect("no global window exists");
        
        // JavaScript to open a file using the File System Access API
        let js_open = r#"
        (async function() {
            try {
                // Check if the File System Access API is supported
                if (!('showOpenFilePicker' in window)) {
                    throw new Error('File System Access API not supported');
                }
                
                const options = {
                    types: [
                        {
                            description: 'Text Files',
                            accept: {'text/plain': ['.txt', '.rs', '.js', '.html', '.css', '.md', '.json', '.toml', '.yaml', '.yml']}
                        }
                    ],
                    multiple: false
                };
                
                const [handle] = await window.showOpenFilePicker(options);
                const file = await handle.getFile();
                const contents = await file.text();
                
                // Store the file handle for later use
                window._openedFileHandle = handle;
                
                // Determine language from extension
                const ext = handle.name.split('.').pop().toLowerCase();
                let lang = 'plain';
                switch (ext) {
                    case 'rs': lang = 'rust'; break;
                    case 'js': lang = 'javascript'; break;
                    case 'html': lang = 'html'; break;
                    case 'css': lang = 'css'; break;
                    case 'md': lang = 'markdown'; break;
                    case 'json': lang = 'json'; break;
                    case 'toml': lang = 'toml'; break;
                    case 'yaml':
                    case 'yml': lang = 'yaml'; break;
                }
                
                return { success: true, name: handle.name, contents, language: lang, handle };
            } catch (e) {
                console.error("Error opening file:", e);
                
                // If File System Access API is not supported, fall back to file input
                if (e.message === 'File System Access API not supported') {
                    return { success: false, fallback: true, error: e.toString() };
                }
                
                return { success: false, error: e.toString() };
            }
        })()
        "#;
        
        // Execute the JavaScript
        let _ = js_sys::eval(js_open);
        
        // Use a script to check results and call back to our Rust code
        let document = window.document().expect("should have a document on window");
        let script = document.create_element("script").expect("couldn't create script");
        
        script.set_text_content(Some(&format!(
            r#"
            (async function() {{
                try {{
                    const result = await {};
                    
                    if (result && result.success) {{
                        // Call back to Rust with the file contents and info
                        window._handleOpenedFile && window._handleOpenedFile(
                            result.contents, 
                            result.name,
                            result.language
                        );
                        
                        // Store file handle
                        window._storeOpenedFileHandle && window._storeOpenedFileHandle(window._openedFileHandle);
                    }} else if (result && result.fallback) {{
                        // Fall back to file input
                        const input = document.createElement('input');
                        input.type = 'file';
                        input.accept = '.txt,.rs,.js,.html,.css,.md,.json,.toml,.yaml,.yml';
                        
                        input.onchange = (event) => {{
                            const file = event.target.files[0];
                            if (!file) return;
                            
                            const reader = new FileReader();
                            reader.onload = (e) => {{
                                const contents = e.target.result;
                                
                                // Determine language from extension
                                const ext = file.name.split('.').pop().toLowerCase();
                                let lang = 'plain';
                                switch (ext) {{
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
                                
                                window._handleOpenedFile && window._handleOpenedFile(
                                    contents, 
                                    file.name,
                                    lang
                                );
                            }};
                            reader.readAsText(file);
                        }};
                        
                        input.click();
                    }}
                }} catch (e) {{
                    console.error("Error processing open result:", e);
                }}
            }})();
            "#,
            js_open
        )));
        
        document.body().expect("no body").append_child(&script).expect("couldn't append script");
        
        // Create callback functions for JavaScript to call
        let handle_opened_file = Closure::wrap(Box::new(move |content: String, name: String, lang: String| {
            buffer.set(Buffer::from_str(&content, Some(name.clone())));
            filename.set(Some(name));
            language.set(Some(lang));
        }) as Box<dyn FnMut(String, String, String)>);
        
        let store_file_handle = Closure::wrap(Box::new(move |handle: web_sys::FileSystemFileHandle| {
            file_handle.set(Some(handle));
        }) as Box<dyn FnMut(web_sys::FileSystemFileHandle)>);
        
        // Attach callbacks to window
        let window_any = window.dyn_into::<web_sys::js_sys::Object>().expect("window should be an object");
        js_sys::Reflect::set(
            &window_any, 
            &JsValue::from_str("_handleOpenedFile"), 
            &handle_opened_file.as_ref()
        ).expect("Failed to set window._handleOpenedFile");
        
        js_sys::Reflect::set(
            &window_any, 
            &JsValue::from_str("_storeOpenedFileHandle"), 
            &store_file_handle.as_ref()
        ).expect("Failed to set window._storeOpenedFileHandle");
        
        // Prevent the callbacks from being dropped
        handle_opened_file.forget();
        store_file_handle.forget();
    });

    let fallback_save_download = {
        let buffer = buffer.clone();
        let filename = filename.clone();
    
        move || {
            let current_text = buffer.read().text();
            let current_filename = filename.read().clone().unwrap_or_else(|| "untitled.txt".to_string());
    
            // Create a Blob and download link
            let js_code = "
                (function() {
                    const blob = new Blob([window._contentToSave], {type: 'text/plain'});
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = window._suggestedFilename;
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                })();
            ";
    
            // Set up global variables for the JavaScript to use
            let window = web_sys::window().expect("no global window exists");
            let window_any = window.dyn_into::<js_sys::Object>().expect("window should be an object");
            
            js_sys::Reflect::set(
                &window_any,
                &JsValue::from_str("_contentToSave"),
                &JsValue::from_str(&current_text)
            ).expect("Failed to set content");
            
            js_sys::Reflect::set(
                &window_any,
                &JsValue::from_str("_suggestedFilename"),
                &JsValue::from_str(&current_filename)
            ).expect("Failed to set filename");
            
            // Execute the JavaScript
            let _ = js_sys::eval(js_code);
        }
    };

    let handle_save_as = use_callback(move |_| {
        let window = web_sys::window().expect("no global window exists");
        let current_text = buffer.read().text();
        let current_filename = filename.read().clone().unwrap_or_else(|| "untitled.txt".to_string());
        
        // Check if File System Access API is supported
        let is_fsapi_supported = js_sys::eval("'showSaveFilePicker' in window")
            .unwrap_or(JsValue::FALSE).as_bool().unwrap_or(false);
        
        if is_fsapi_supported {
            // Store content and filename in global variables first
            let window_any = window.dyn_into::<js_sys::Object>().expect("window should be an object");
            
            js_sys::Reflect::set(
                &window_any,
                &JsValue::from_str("_contentToSave"),
                &JsValue::from_str(&current_text)
            ).expect("Failed to set content");
            
            js_sys::Reflect::set(
                &window_any,
                &JsValue::from_str("_suggestedFilename"),
                &JsValue::from_str(&current_filename)
            ).expect("Failed to set filename");
            
            // Set up our callbacks
            let update_info = Closure::wrap(Box::new(move |name: String, lang: String| {
                filename.set(Some(name));
                language.set(Some(lang));
            }) as Box<dyn FnMut(String, String)>);
            
            let store_handle = Closure::wrap(Box::new(move |handle: web_sys::FileSystemFileHandle| {
                file_handle.set(Some(handle));
            }) as Box<dyn FnMut(web_sys::FileSystemFileHandle)>);
            
            js_sys::Reflect::set(
                &window_any, 
                &JsValue::from_str("_updateFileInfo"), 
                &update_info.as_ref()
            ).expect("Failed to set update callback");
            
            js_sys::Reflect::set(
                &window_any, 
                &JsValue::from_str("_storeFileHandle"), 
                &store_handle.as_ref()
            ).expect("Failed to set store handle callback");
            
            // Single JavaScript code block
            let js_code = "
                (async function() {
                    try {
                        const options = {
                            suggestedName: window._suggestedFilename || 'untitled.txt',
                            types: [{
                                description: 'Text Files',
                                accept: {'text/plain': ['.txt', '.rs', '.js', '.html', '.css', '.md', '.json', '.toml', '.yaml', '.yml']}
                            }]
                        };
                        
                        const handle = await window.showSaveFilePicker(options);
                        const writable = await handle.createWritable();
                        await writable.write(window._contentToSave || '');
                        await writable.close();
                        
                        window._savedFileHandle = handle;
                        
                        // Determine language from extension
                        const ext = handle.name.split('.').pop().toLowerCase();
                        let lang = 'plain';
                        switch (ext) {
                            case 'rs': lang = 'rust'; break;
                            case 'js': lang = 'javascript'; break;
                            case 'html': lang = 'html'; break;
                            case 'css': lang = 'css'; break;
                            case 'md': lang = 'markdown'; break;
                            case 'json': lang = 'json'; break;
                            case 'toml': lang = 'toml'; break;
                            case 'yaml': case 'yml': lang = 'yaml'; break;
                        }
                        
                        if (window._updateFileInfo) {
                            window._updateFileInfo(handle.name, lang);
                        }
                        if (window._storeFileHandle) {
                            window._storeFileHandle(handle);
                        }
                    } catch (err) {
                        console.error('Error in save as:', err);
                    }
                })();
            ";
            
            // Execute the JavaScript
            let _ = js_sys::eval(js_code);
            
            // Prevent callbacks from being dropped
            update_info.forget();
            store_handle.forget();
        } else {
            // Firefox fallback: Direct download
            fallback_save_download();
        }
    });
    
    let handle_save_file = use_callback(move |_| {
        let window = web_sys::window().expect("no global window exists");
        let current_text = buffer.read().text();
        
        // Check if File System Access API is supported and we have a file handle
        let is_fsapi_supported = js_sys::eval("'showSaveFilePicker' in window")
            .unwrap_or(JsValue::FALSE).as_bool().unwrap_or(false);
        
        if is_fsapi_supported && file_handle.read().is_some() {
            // Set up the content to save
            let window_any = window.dyn_into::<js_sys::Object>().expect("window should be an object");
            js_sys::Reflect::set(
                &window_any,
                &JsValue::from_str("_contentToSave"),
                &JsValue::from_str(&current_text)
            ).expect("Failed to set content");
            
            // Single JavaScript code block
            let js_code = "
                (async function() {
                    try {
                        const handle = window._savedFileHandle;
                        if (!handle) {
                            throw new Error('No file handle available');
                        }
                        
                        const writable = await handle.createWritable();
                        await writable.write(window._contentToSave || '');
                        await writable.close();
                        return true;
                    } catch (err) {
                        console.error('Error saving file:', err);
                        return false;
                    }
                })();
            ";
            
            // Execute the JavaScript
            let _ = js_sys::eval(js_code);
        } else {
            // No file handle or API not supported, do Save As
            handle_save_as(());
        }
    });

    // Get current theme
    let current_theme = &themes[current_theme_idx()];

    // Set up global JavaScript handlers to bridge between menu and component
let setup_js_handlers = {
    let handle_new_file = handle_new_file.clone();
    let handle_open_file = handle_open_file.clone();
    let handle_save_file = handle_save_file.clone();
    let handle_save_as = handle_save_as.clone();
    let current_theme_idx = current_theme_idx.clone();
    let themes = themes.clone();
    
    move || {
        // Create handler for new file
        let new_file_handler = Closure::wrap(Box::new(move || {
            handle_new_file(());
        }) as Box<dyn FnMut()>);
        
        // Create handler for open file
        let open_file_handler = Closure::wrap(Box::new(move || {
            handle_open_file(());
        }) as Box<dyn FnMut()>);
        
        // Create handler for save
        let save_handler = Closure::wrap(Box::new(move || {
            handle_save_file(());
        }) as Box<dyn FnMut()>);
        
        // Create handler for save as
        let save_as_handler = Closure::wrap(Box::new(move || {
            handle_save_as(());
        }) as Box<dyn FnMut()>);
        
        // Create handler for theme change
        let theme_handler = {
            let mut current_theme_idx = current_theme_idx.clone();
            let themes = themes.clone();
            
            Closure::wrap(Box::new(move |theme_type: String| {
                let target_substring = if theme_type == "light" { "Light" } else { "Dark" };
                if let Some(idx) = themes.iter().position(|theme| theme.name.contains(target_substring)) {
                    current_theme_idx.set(idx);
                }
            }) as Box<dyn FnMut(String)>)
        };
        
        // Get window
        let window = web_sys::window().expect("no global window exists");
        let window_any = window.dyn_into::<js_sys::Object>().expect("window should be an object");
        
        // Create the actions object
        let actions = js_sys::Object::new();
        
        // Set the handlers
        js_sys::Reflect::set(
            &actions, 
            &JsValue::from_str("newFile"), 
            &new_file_handler.as_ref()
        ).expect("Failed to set newFile handler");
        
        js_sys::Reflect::set(
            &actions, 
            &JsValue::from_str("openFile"), 
            &open_file_handler.as_ref()
        ).expect("Failed to set openFile handler");
        
        js_sys::Reflect::set(
            &actions, 
            &JsValue::from_str("saveFile"), 
            &save_handler.as_ref()
        ).expect("Failed to set saveFile handler");
        
        js_sys::Reflect::set(
            &actions, 
            &JsValue::from_str("saveFileAs"), 
            &save_as_handler.as_ref()
        ).expect("Failed to set saveFileAs handler");
        
        js_sys::Reflect::set(
            &actions, 
            &JsValue::from_str("setTheme"), 
            &theme_handler.as_ref()
        ).expect("Failed to set setTheme handler");
        
        // Set the actions object on window
        js_sys::Reflect::set(
            &window_any,
            &JsValue::from_str("_editorActions"),
            &actions
        ).expect("Failed to set _editorActions on window");
        
        // Prevent handlers from being dropped
        new_file_handler.forget();
        open_file_handler.forget();
        save_handler.forget();
        save_as_handler.forget();
        theme_handler.forget();
    }
};

// Call the setup function
setup_js_handlers();

// Create menu handler with current state
let menu_handler = EditorMenuHandler::new(
    buffer.read().is_modified(),
    filename.read().is_some(),
    themes[current_theme_idx()].name.contains("Light"),
);

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100vh; overflow: hidden;",
            MenuBar {
                theme: current_theme.clone(),
                menus: menu_items,
                handler: menu_handler,
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

            // Conditionally render the NewFileDialog when show_new_file_dialog is true/false
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