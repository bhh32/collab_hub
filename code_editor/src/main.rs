mod code_editor;
mod code_editor_view;
mod highlighter;

use dioxus::prelude::*;
use crate::code_editor::CodeEditor;

fn main() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();

    // Launch the web app
    launch(App);
}

#[component]
fn App() -> Element {
    // Render the CodeEditor component (which should handle everything else)
    rsx! {
        CodeEditor {}
    }
}