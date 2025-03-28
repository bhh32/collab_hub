use dioxus::prelude::*;
use crate::core::Theme;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

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

// Component for rendering a nested submenu
#[component]
fn NestedSubmenu<H: MenuHandler + Clone + PartialEq + 'static>(
    theme: Theme,
    submenu: Vec<MenuItem>,
    parent_id: String,
    handler: H,
    dropdown_item_style: String,
    disabled_style: String,
) -> Element {
    let container_style = format!(
        "position: absolute; left: 100%; top: 0; background-color: {}; color: {}; \
         min-width: 200px; box-shadow: 0 2px 5px rgba(0, 0, 0, 0.3); z-index: 1000; \
         display: none; flex-direction: column; padding: 0.25rem 0;",
        theme.ui.toolbar_bg, theme.ui.toolbar_fg
    );

    rsx! {
        div {
            class: "submenu-container",
            "data-submenu-id": "{parent_id}",
            style: container_style,
            
            {
                submenu.iter().map(|item| {
                    let item_id = item.id.clone();
                    let item_id_clone = item_id.clone();
                    let item_label = item.label.clone();
                    let is_enabled = handler.is_item_enabled(&item_id);
                    let is_checked = handler.is_item_checked(&item_id);
                    let has_shortcut = item.shortcut.is_some();
                    let has_submenu = item.submenu.is_some();
                    let is_action = item.action;
                    
                    let item_style = format!("{} {}", dropdown_item_style, 
                                         if !is_enabled { &disabled_style } else { "" });

                    let mut handler_clone = handler.clone();
                    
                    rsx! {
                        div {
                            key: "{item_id.clone()}",
                            "data-menu-id": "{item_id.clone()}",
                            style: item_style,
                            onclick: move |event: MouseEvent| {
                                if !is_enabled {
                                    event.stop_propagation();
                                    return;
                                }

                                if is_action {
                                    handler_clone.handle_menu_action(&item_id_clone.clone());
                                    event.stop_propagation();
                                }
                            },
                            
                            // Left side with checkbox and label
                            div {
                                style: "display: flex; align-items: center;",
                                
                                // Show checkbox if applicable
                                if let Some(checked) = is_checked {
                                    span {
                                        style: "margin-right: 0.5rem; width: 1rem;",
                                        {
                                            if checked {
                                                "✓"
                                            } else {
                                                " "
                                            }
                                        }
                                    }
                                }
                                
                                // Item label
                                span { {item_label.clone()} }
                            }
                            
                            // Right side with shortcut
                            div {
                                style: "display: flex; align-items: center;",
                                
                                if let Some(shortcut) = &item.shortcut {
                                    span {
                                        style: "color: #999; font-size: 0.9em; margin-left: 1rem",
                                        {shortcut.clone()}
                                    }
                                }
                                
                                // Show submenu indicator if it has nested submenu
                                if has_submenu {
                                    span {
                                        style: "margin-left: 0.5rem;",
                                        "▶"
                                    }
                                }
                            }
                        }
                    }
                })
            }
        }
    }
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
    
    // Set up global JS handler for nested menu items
    let click_handler = {
        let mut handler_clone = handler.clone();
        let mut active_menu_clone = active_menu.clone();
        
        Closure::wrap(Box::new(move |action_id: String| {
            // Close the menu
            active_menu_clone.set(None);
            
            // Call the action handler
            handler_clone.handle_menu_action(&action_id);
        }) as Box<dyn FnMut(String)>)
    };
    
    // Attach to window
    let window = web_sys::window().expect("no global window exists");
    let window_obj = window.dyn_into::<js_sys::Object>().expect("window should be an object");
    
    js_sys::Reflect::set(
        &window_obj,
        &JsValue::from_str("_handleMenuAction"),
        &click_handler.as_ref()
    ).expect("Failed to set menu handler");
    
    // Prevent the callback from being dropped
    click_handler.forget();
    
    // Set up general menu event handlers using JavaScript
    use_effect(move || {
        let menu_js = r#"
            // Setup function to handle menu events
            function setupMenuEvents() {
                // Handle clicks on menu items
                document.querySelectorAll('[data-menu-id]').forEach(item => {
                    // Click handler for menu actions
                    item.addEventListener('click', event => {
                        if (window._handleMenuAction) {
                            window._handleMenuAction(item.getAttribute('data-menu-id'));
                        }
                        event.stopPropagation();
                    });
                    
                    // Hover handler for menu navigation
                    item.addEventListener('mouseover', event => {
                        // Hide all other submenus at this level
                        const parentMenu = item.closest('.submenu-container');
                        if (parentMenu) {
                            const siblings = parentMenu.querySelectorAll('.submenu-container');
                            siblings.forEach(menu => {
                                menu.style.display = 'none';
                            });
                        }
                        
                        // Show this item's submenu if it has one
                        const submenuId = item.getAttribute('data-has-submenu');
                        if (submenuId) {
                            const submenu = document.querySelector(`[data-submenu-id="${submenuId}"]`);
                            if (submenu) {
                                submenu.style.display = 'flex';
                            }
                        }
                    });
                });
            }
            
            // Run the setup
            setupMenuEvents();
            
            // Set up a MutationObserver to handle dynamically added menu items
            const menuObserver = new MutationObserver(mutations => {
                setupMenuEvents();
            });
            
            // Observe the entire document for changes to the DOM
            menuObserver.observe(document.body, { 
                childList: true,
                subtree: true
            });
        "#;
        
        let _ = js_sys::eval(menu_js);
        
        // Cleanup on unmount
        (move || {
            let _ = js_sys::eval(r#"
                // Clean up the observer when menu is unmounted
                if (window.menuObserver) {
                    window.menuObserver.disconnect();
                }
            "#);
        })()
    });

    // Render the menu bar
    rsx! {
        div {
            style: menu_bar_style,
            onmousedown: move |_| {
                // This prevents text selection when clicking the menu
                let _ = js_sys::eval("document.getSelection().removeAllRanges();");
            },

            // Render top-level menu items
            {
                menus.iter().map(|item| {
                    let item_id = item.id.clone();
                    let item_id_onmouseover = item_id.clone();
                    let item_id_onclick = item_id.clone();
                    let item_label = item.label.clone();
                    let has_submenu = item.submenu.is_some();
                    let is_active = active_menu() == Some(item_id.clone());
                    let item_style = format!("{} {}", menu_item_style, 
                                          if is_active { &menu_item_hover_style } else { "" });
                    let mut active_menu_clone = active_menu.clone();
                    
                    rsx! {
                        div {
                            key: item_id.clone(),
                            style: item_style,
                            onmouseover: move |_| {
                                // If a menu is already open, switch to this one immediately on hover
                                if active_menu_clone().is_some() {
                                    // Close any open submenus first
                                    let _ = js_sys::eval("document.querySelectorAll('.submenu-container').forEach(m => m.style.display = 'none');");
                                    // Set the new active menu
                                    active_menu_clone.set(Some(item_id_onmouseover.clone()));
                                }
                            },
                            onclick: move |_| {
                                toggle_menu(item_id_onclick.clone());
                            },
                            
                            // Item label
                            span { {item_label.clone()} }
                            
                            // Render dropdown if this menu is active
                            if is_active && has_submenu {
                                div {
                                    style: dropdown_style.clone(),
                                    onclick: move |event| { event.stop_propagation(); },
                                    
                                    {
                                        item.submenu.as_ref().unwrap().iter().map(|submenu_item| {
                                            let sub_id = submenu_item.id.clone();
                                            let sub_label = submenu_item.label.clone();
                                            let is_enabled = handler.is_item_enabled(&sub_id);
                                            let is_checked = handler.is_item_checked(&sub_id);
                                            let has_shortcut = submenu_item.shortcut.is_some();
                                            let shortcut = submenu_item.shortcut.clone();
                                            let sub_style = format!("{} {}", dropdown_item_style, 
                                                               if !is_enabled { disabled_style } else { "" });
                                            let mut on_action = handle_menu_action.clone();
                                            let is_action = submenu_item.action;
                                            
                                            // Check if this submenu item has its own submenu
                                            let has_nested_submenu = submenu_item.submenu.is_some();
                                            
                                            rsx! {
                                                div {
                                                    key: sub_id.clone(),
                                                    style: sub_style,
                                                    "attr:data_menu_id": sub_id.clone(),
                                                    "attr:data_has_submenu": if has_nested_submenu { Some(sub_id.clone()) } else { None },
                                                    // Track hover state to handle submenu display
                                                    onmouseover: {
                                                        let sub_id_for_hover = sub_id.clone();
                                                        move |event: dioxus::events::MouseEvent| {
                                                            // If this item has a submenu, we want to show it on hover
                                                            if has_nested_submenu {
                                                                // Stop propagation to prevent parent handlers from firing
                                                                event.stop_propagation();
                                                                
                                                                // Tell JavaScript to show this submenu
                                                                let js_code = format!(
                                                                    "document.querySelectorAll('.submenu-container').forEach(m => m.style.display = 'none'); \
                                                                    const current = document.querySelector('[data-submenu-id=\"{}\"]'); \
                                                                    if (current) current.style.display = 'flex';",
                                                                    sub_id_for_hover
                                                                );
                                                                let _ = js_sys::eval(&js_code);
                                                            }
                                                        }
                                                    },
                                                    onclick: {
                                                        let sub_id_for_click = sub_id.clone();
                                                        move |event: dioxus::events::MouseEvent| {
                                                            if !is_enabled {
                                                                event.stop_propagation();
                                                                return;
                                                            }
                                                            
                                                            if is_action {
                                                                on_action(&sub_id_for_click);
                                                            }
                                                        }
                                                    },
                                                    
                                                    // Left side with checkbox and label
                                                    div {
                                                        style: "display: flex; align-items: center;",
                                                        
                                                        // Show checkbox if applicable
                                                        if let Some(checked) = is_checked {
                                                            span {
                                                                style: "margin-right: 0.5rem; width: 1rem;",
                                                                {
                                                                    if checked {
                                                                        "\u{2713}"
                                                                    } else {
                                                                        " "
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        
                                                        // Item label
                                                        span { {sub_label.clone()} }
                                                    }
                                                    
                                                    div {
                                                        style: "display: flex; align-items: center;",
                                                        
                                                        // Right side with shortcut
                                                        if has_shortcut {
                                                            span {
                                                                style: "color: #999; font-size: 0.9em; margin-left: 1rem",
                                                                {shortcut.clone().unwrap()}
                                                            }
                                                        }
                                                        
                                                        // Show submenu indicator if it has nested submenu
                                                        if has_nested_submenu {
                                                            span {
                                                                style: "margin-left: 0.5rem;",
                                                                "\u{25b6}"
                                                            }
                                                        }
                                                    }
                                                    
                                                    // Include nested submenu if this item has one
                                                    if has_nested_submenu {
                                                        NestedSubmenu {
                                                            theme: theme.clone(),
                                                            submenu: submenu_item.submenu.as_ref().unwrap().clone(),
                                                            parent_id: sub_id.clone(),
                                                            handler: handler.clone(),
                                                            dropdown_item_style: dropdown_item_style.to_string(),
                                                            disabled_style: disabled_style.to_string(),
                                                        }
                                                    }
                                                }
                                            }
                                        })
                                    }
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