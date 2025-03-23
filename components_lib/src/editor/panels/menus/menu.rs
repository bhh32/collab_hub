use dioxus::prelude::*;
use crate::core::Theme;
use std::collections::HashMap;

// Menu item structure
#[derive(Clone, PartialEq)]
pub struct MenuItem {
    pub id: String,                       // Unique identifier for the new menu item
    pub label: String,                    // Display label
    pub action: bool,                     // Is this an action item (vs a submenu header)
    pub submenu: Option<Vec<MenuItem>>,
    pub shortcut: Option<String>,
    pub enabled: bool,                    // Whether the item is enabled
    pub checked: Option<bool>,            // For checkable menu items
}

/// Mandatory handlers for all menus
pub trait MenuHandler {
    fn handle_menu_action(&mut self, action_id: &str);
    fn is_item_enabled(&self, item_id: &str) -> bool;
    fn is_item_checked(&self, item_id: &str) -> Option<bool>;
}

#[component]
pub fn MenuBar<H: MenuHandler + Clone + PartialEq + 'static> (
    theme: Theme,
    menus: Vec<MenuItem>,
    handler: H,
) -> Element {
    // Track which menu is currently open
    let mut active_menu = use_signal(|| None::<String>);

    // Styles for the menu bar
    let menu_bar_style = format!(
        "display: flex; background-color: {}; color: {}; padding: 0;",
        theme.ui.toolbar_bg, theme.ui.toolbar_fg
    );

    // Style for menu items
    let menu_item_style = "padding: 0.25rem 0.7rem; cursor: pointer; position: relative;";
    let menu_item_hover_style = format!(
        "background-color: {};",
        theme.ui.button_hover
    );

    // Style for dropdown menus
    let dropdown_style = format!(
        "position: absolute; top: 100%; left: 0; background-color: {}; color: {}; \
         min-width: 200px; box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3); z-index: 1000; \
         display: flex; flex-direction: column; padding: 0.25rem 0;",
         theme.ui.toolbar_bg, theme.ui.toolbar_fg
    );

    // Style for dropdown menu items
    let dropdown_item_style = "padding: 0.5rem 1rem; display: flex; justify-content: space-between; cursor: pointer;";
    
    // Style for disabled items
    let disabled_style = "opacity: 0.5; cursor: default;";

    // Handle toggling a menu
    let mut toggle_menu = move |menu_id: String| {
        if active_menu() == Some(menu_id.clone()) {
            active_menu.set(None);
        } else {
            active_menu.set(Some(menu_id));
        }
    };

    // Handle clicking a menu item
    let handle_menu_action = {
        let mut handler = handler.clone();

        move |item_id: &str| {
            // Close the menu
            active_menu.set(None);

            // Call the action handler
            handler.handle_menu_action(item_id);
        }
    };

    rsx! {
        div {
            style: menu_bar_style,
            onmousedown: move |_| {
                // This prevents text selection when clicking the menu
                let _ = js_sys::eval("document.getSelection().removeAllRanges();");
            },

            // Render top-level menu items
            {
                menus.clone().into_iter().map(|item| {
                    let item_id = item.id.clone();
                    let item_label = item.label.clone();
                    let has_submenu = item.submenu.is_some();
                    let is_active = active_menu() == Some(item_id.clone());
                    
                    let style_value = if is_active {
                        format!("{} {}", menu_item_style, menu_item_hover_style)
                    } else {
                        menu_item_style.to_string()
                    };

                    let item_id_clone = item_id.clone();
                    
                    rsx! {
                        div {
                            key: "{item_id.clone()}",
                            style: style_value,
                            onmouseover: move |_| {
                                // If a menu is already open, switch to this one
                                if active_menu().is_some() {
                                    active_menu.set(Some(item_id_clone.clone()));
                                }
                            },
                            onclick: move |_| {
                                toggle_menu(item_id.clone());
                            },
                            
                            // Item label
                            span { "{item_label}" }
                        }
                        
                        // Render dropdown if this menu is active
                        if is_active && has_submenu {
                            div {
                                style: dropdown_style.clone(),
                                onclick: move |event| event.stop_propagation(),
                                
                                {
                                    item.submenu.clone().unwrap().into_iter().map(|submenu_item| {
                                        let sub_id = submenu_item.id.clone();
                                        let sub_label = submenu_item.label.clone();
                                        let is_enabled = handler.is_item_enabled(&sub_id);
                                        let is_checked = handler.is_item_checked(&sub_id);
                                        
                                        let sub_style = if is_enabled {
                                            dropdown_item_style.to_string()
                                        } else {
                                            format!("{} {}", dropdown_item_style, disabled_style)
                                        };
                                        
                                        let mut handle_action = handle_menu_action.clone();
                                        
                                        rsx! {
                                            div {
                                                key: "{sub_id}",
                                                style: sub_style,
                                                onclick: move |event| {
                                                    if !is_enabled {
                                                        event.stop_propagation();
                                                        return;
                                                    }
                                                    
                                                    if submenu_item.action {
                                                        handle_action(&sub_id);
                                                    }
                                                },
                                                
                                                // Left side - label with possible checkbox
                                                div {
                                                    style: "display: flex; align-items: center;",
                                                    
                                                    if let Some(checked) = is_checked {
                                                        span {
                                                            style: "margin-right: 0.5rem; width: 1rem;",
                                                            if checked { "✓" } else { " " }
                                                        }
                                                    }
                                                    
                                                    span { "{sub_label}" }
                                                }
                                                
                                                // Right side - shortcut text
                                                if let Some(shortcut) = &submenu_item.shortcut {
                                                    div {
                                                        style: "color: #999; font-size: 0.9em; margin-left: 1rem;",
                                                        "{shortcut}"
                                                    }
                                                }
                                            }
                                        }
                                    })
                                }
                            }
                        }
                    }
                })
            }
        }

        // Add an invisible overlay to close menus when clicking elsewhere
        if active_menu().is_some() {
            div {
                style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; z-index: 999;",
                onclick: move |_| active_menu.set(None),
            }
        }
    }
}