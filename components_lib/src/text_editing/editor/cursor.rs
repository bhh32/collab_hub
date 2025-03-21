#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CursorPosition {
    pub offset: usize,
    pub line: usize,
    pub column: usize,
}