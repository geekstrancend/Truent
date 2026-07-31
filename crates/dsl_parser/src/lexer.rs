//! Lexer for the Truent DSL (uses pest internally).

/// Token type placeholder; pest handles tokenization.
/// This module is included for future extensibility.
pub struct Token {
    /// Token type.
    pub token_type: TokenType,
    /// Source position (line, col).
    pub position: (usize, usize),
}

/// Token types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    /// Invariant keyword.
    Invariant,
    /// Identifier.
    Identifier(String),
    /// Integer literal.
    Integer(i128),
    /// Boolean literal.
    Boolean(bool),
    /// Operator.
    Operator(String),
    /// Left brace.
    LeftBrace,
    /// Right brace.
    RightBrace,
    /// Left paren.
    LeftParen,
    /// Right paren.
    RightParen,
    /// Comma.
    Comma,
    /// End of file.
    Eof,
}
