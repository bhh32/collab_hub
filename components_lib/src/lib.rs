pub mod editor;
pub mod core;

// Rexport the editor component
pub use editor::CodeEditor;
pub use crate::core::themes::{Theme, available_themes};
