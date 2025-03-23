use dioxus::prelude::*;
use crate::core::Theme;

#[component]
pub fn StatusBar(
    theme: Theme,
    filename: Option<String>,
    language: Option<String>,
    cursor_line: usize,
    cursor_column: usize,
    total_lines: usize,
) -> Element {
    let style = format!(
        "display: flex; padding: 0.25rem 0.5rem; font-size: 12px;
         background-color: {}; color: {};",
         theme.ui.statusbar_bg, theme.ui.statusbar_fg
    );

    let display_filename = filename.clone().unwrap_or_else(|| "untitled".to_string());
    let display_language = language.clone().unwrap_or_else(|| "plain text".to_string());

    rsx! {
        div {
            style: style,
            div {
                style: "flex: 1;",
                "{display_filename} - {display_language}"
            }
            div {
                "Ln {cursor_line + 1}, Col {cursor_column + 1} | {total_lines} lines"
            }
        }
    }
}