use ropey::Rope;
use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct Buffer {
    rope: Arc<Rope>,
    modified: bool,
    filename: Option<String>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            rope: Arc::new(Rope::new()),
            modified: false,
            filename: None,
        }
    }

    pub fn from_str(content: &str, filename: Option<String>) -> Self {
        Self {
            rope: Arc::new(Rope::from_str(content)),
            modified: false,
            filename,
        }
    }

    pub fn insert(&mut self, char_idx: usize, text: &str) -> Result<(), String> {
        if char_idx <= self.rope.len_chars() {
            let mut new_rope = (*self.rope).clone();
            new_rope.insert(char_idx, text);
            self.rope = Arc::new(new_rope);
            self.modified = true;
            Ok(())
        } else {
            Err("Character index out of bounds".to_string())
        }
    }

    pub fn delete(&mut self, char_idx: usize, len: usize) -> Result<(), String> {
        if char_idx + len <= self.rope.len_chars() {
            let mut new_rope = (*self.rope).clone();
            new_rope.remove(char_idx..(char_idx + len));
            self.rope = Arc::new(new_rope);
            self.modified = true;
            Ok(())
        } else {
            Err("Delete range out of bounds".to_string())
        }
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn line(&self, idx: usize) -> Option<String> {
        if idx < self.rope.len_lines() {
            Some(self.rope.line(idx).to_string())
        } else {
            None
        }
    }

    pub fn filename(&self) -> Option<&String> {
        self.filename.as_ref()
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
}