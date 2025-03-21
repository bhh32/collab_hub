use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub selection: String,
    pub cursor: String,
    pub line_highlight: String,
    pub syntax_colors: HashMap<String, String>,
    pub ui: UiColors,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiColors {
    pub toolbar_bg: String,
    pub toolbar_fg: String,
    pub statusbar_bg: String,
    pub statusbar_fg: String,
    pub button: String,
    pub button_hover: String,
    pub button_active: String,
}

impl Default for Theme {
    fn default() -> Self {
        let mut syntax_colors = HashMap::new();
        syntax_colors.insert("keyword".to_string(), "#C678DD".to_string());
        syntax_colors.insert("string".to_string(), "#98C379".to_string());
        syntax_colors.insert("comment".to_string(), "#7F848E".to_string());
        syntax_colors.insert("function".to_string(), "#61AFEF".to_string());
        syntax_colors.insert("type".to_string(), "#E5C07B".to_string());

        Self {
            name: "Default Dark".to_string(),
            background: "#282C34".to_string(),
            foreground: "#ABB2BF".to_string(),
            selection: "#3E4451".to_string(),
            cursor: "#528BFF".to_string(),
            line_highlight: "#2C313A".to_string(),
            syntax_colors,
            ui: UiColors {
                toolbar_bg: "#21252B".to_string(),
                toolbar_fg: "#ABB2BF".to_string(),
                statusbar_bg: "#21252B".to_string(),
                statusbar_fg: "#9DA5B4".to_string(),
                button: "#3A3F4B".to_string(),
                button_hover: "#4B5263".to_string(),
                button_active: "#528BFF".to_string(),
            },
        }
    }
}

pub fn light_theme() -> Theme {
    let mut light_theme = Theme::default();
    light_theme.name = "Light".to_string();
    light_theme.background = "#FFFFFF".to_string();
    light_theme.foreground = "#383A42".to_string();
    light_theme.selection = "#E5E5E6".to_string();
    light_theme.cursor = "#526FFF".to_string();
    light_theme.line_highlight = "#F2F2F2".to_string();
    
    let mut syntax_colors = HashMap::new();
    syntax_colors.insert("keyword".to_string(), "#A626A4".to_string());
    syntax_colors.insert("string".to_string(), "#50A14F".to_string());
    syntax_colors.insert("comment".to_string(), "#A0A1A7".to_string());
    syntax_colors.insert("function".to_string(), "#4078F2".to_string());
    syntax_colors.insert("type".to_string(), "#C18401".to_string());
    light_theme.syntax_colors = syntax_colors;
    
    light_theme.ui = UiColors {
        toolbar_bg: "#E5E5E6".to_string(),
        toolbar_fg: "#383A42".to_string(),
        statusbar_bg: "#E5E5E6".to_string(),
        statusbar_fg: "#696C77".to_string(),
        button: "#D4D4D4".to_string(),
        button_hover: "#CACACA".to_string(),
        button_active: "#4078F2".to_string(),
    };

    light_theme
}

pub fn available_themes() -> Vec<Theme> {
    vec![Theme::default(), light_theme()]
}

impl Theme {
    pub fn get_color(&self, token_type: &str) -> String {
        match token_type {
            "keyword" => self.syntax_colors.get("keyword").cloned().unwrap_or_else(|| "#C678DD".to_string()),
            "string" => self.syntax_colors.get("string").cloned().unwrap_or_else(|| "#98C379".to_string()),
            "comment" => self.syntax_colors.get("comment").cloned().unwrap_or_else(|| "#7F848E".to_string()),
            "function" => self.syntax_colors.get("function").cloned().unwrap_or_else(|| "#61AFEF".to_string()),
            "type" => self.syntax_colors.get("type").cloned().unwrap_or_else(|| "#E5C07B".to_string()),
            "number" => self.syntax_colors.get("number").cloned().unwrap_or_else(|| "#D19A66".to_string()),
            _ => self.foreground.clone(),
        }
    }
}