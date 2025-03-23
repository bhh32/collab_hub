use dioxus::prelude::*;
use components_lib::editor::CodeEditor;

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