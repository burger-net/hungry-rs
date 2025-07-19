use std::fmt;
use std::ops::Range;

mod iter;
mod kind;

pub(super) use iter::Tokens;
pub use kind::TokenKind;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownToken(pub Range<usize>);

impl fmt::Display for UnknownToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown token in range {:?}", self.0)
    }
}

impl std::error::Error for UnknownToken {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} token at position {}", self.kind, self.pos)
    }
}
