mod code_editor;
mod code_editor_view;
mod highlighter;

use dioxus::{prelude::*, web::{launch::launch_cfg, Config}};
use crate::code_editor::CodeEditor;

fn main() {
    launch_cfg(App, Config::new().rootname("/code_editor"));
}

#[component]
pub fn App() -> Element {
    // Render the CodeEditor component (which should handle everything else)
    rsx! {
        CodeEditor {}
    }
}