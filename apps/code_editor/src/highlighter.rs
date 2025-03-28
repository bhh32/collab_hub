// src/text_editing/editor/highlighter.rs
use components_lib::core::themes::Theme;
use std::collections::HashMap;

pub struct SyntaxHighlighter {
    language: String,
    theme: Theme,
    keyword_patterns: HashMap<String, Vec<&'static str>>,
}

impl SyntaxHighlighter {
    pub fn new(language: String, theme: Theme) -> Self {
        let mut keyword_patterns = HashMap::new();
        
        // Rust keywords
        keyword_patterns.insert("rust".to_string(), vec![
            "fn", "let", "mut", "pub", "impl", "struct", "enum", "trait", "use", "mod",
            "match", "if", "else", "for", "while", "loop", "return", "self", "super", "where"
        ]);
        
        // JavaScript keywords
        keyword_patterns.insert("javascript".to_string(), vec![
            "function", "var", "let", "const", "class", "import", "export", "from", "return",
            "if", "else", "for", "while", "switch", "case", "default", "break", "continue"
        ]);
        
        Self {
            language,
            theme,
            keyword_patterns,
        }
    }
    
    pub fn highlight(&self, text: &str) -> String {
        let mut result = String::new();
        let lines = text.split('\n');
        
        for line in lines {
            let highlighted_line = self.highlight_line(line);
            result.push_str(&highlighted_line);
            result.push_str("\n");
        }
        
        result
    }
    
    fn highlight_line(&self, line: &str) -> String {
        // Simple syntax highlighting by word
        let mut result = String::new();
        let mut in_string = false;
        let in_comment = false;
        let mut current_word = String::new();
        
        // Check for comments first (simplest case)
        if line.trim().starts_with("//") {
            return format!("<span style=\"color: {}\">{}</span>", 
                self.theme.get_color("comment"), line);
        }
        
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            let c = chars[i];
            
            // Handle strings (simplistic approach)
            if c == '"' && !in_comment {
                if in_string {
                    current_word.push(c);
                    result.push_str(&format!("<span style=\"color: {}\">{}</span>", 
                        self.theme.get_color("string"), current_word));
                    current_word = String::new();
                    in_string = false;
                } else {
                    if !current_word.is_empty() {
                        self.add_highlighted_word(&mut result, &current_word);
                        current_word = String::new();
                    }
                    current_word.push(c);
                    in_string = true;
                }
            } 
            // Handle comments
            else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' && !in_string {
                if !current_word.is_empty() {
                    self.add_highlighted_word(&mut result, &current_word);
                    current_word = String::new();
                }
                // Add the rest of the line as a comment
                let comment = &line[i..];
                result.push_str(&format!("<span style=\"color: {}\">{}</span>", 
                    self.theme.get_color("comment"), comment));
                break;
            }
            // Handle word boundaries
            else if in_string {
                current_word.push(c);
            }
            else if c.is_alphanumeric() || c == '_' {
                current_word.push(c);
            }
            else {
                if !current_word.is_empty() {
                    self.add_highlighted_word(&mut result, &current_word);
                    current_word = String::new();
                }
                // Special handling for parentheses and brackets
                if c == '(' || c == ')' || c == '{' || c == '}' || c == '[' || c == ']' {
                    result.push_str(&format!("<span style=\"color: {}\">{}</span>", 
                        self.theme.get_color("bracket"), c));
                } else {
                    result.push(c);
                }
            }
            
            i += 1;
        }
        
        // Handle any remaining word
        if !current_word.is_empty() {
            self.add_highlighted_word(&mut result, &current_word);
        }
        
        result
    }
    
    fn add_highlighted_word(&self, result: &mut String, word: &str) {
        // Check if word is a keyword for the current language
        if let Some(keywords) = self.keyword_patterns.get(&self.language) {
            if keywords.contains(&word) {
                result.push_str(&format!("<span style=\"color: {}\">{}</span>", 
                    self.theme.get_color("keyword"), word));
                return;
            }
        }
        
        // Check if word is a number
        if word.parse::<f64>().is_ok() {
            result.push_str(&format!("<span style=\"color: {}\">{}</span>", 
                self.theme.get_color("number"), word));
            return;
        }
        
        // Regular word
        result.push_str(word);
    }
}