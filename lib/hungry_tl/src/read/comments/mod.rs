use std::fmt;
use std::ops::Range;

mod iter;

#[cfg(test)]
mod tests;

pub const LINE_START: &str = "//";
pub const LINE_END: &str = "\n";
pub const BLOCK_START: &str = "/*";
pub const BLOCK_END: &str = "*/";

pub(super) struct Comments<'a> {
    buf: &'a str,
    pos: usize,
    var: Option<Variant>,
}

impl<'a> Comments<'a> {
    pub(super) fn new(buf: &'a str) -> Self {
        Self {
            buf,
            pos: 0,
            var: None,
        }
    }

    pub(super) fn buf(&self) -> &'a str {
        self.buf
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(super) enum Either {
    Content { range: Range<usize> },
    Comment { comment: Comment, ended: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    pub variant: Variant,
    pub range: Range<usize>,
}

impl fmt::Display for Comment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} comment in range {:?}", self.variant, self.range)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Line,
    Block,
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Variant::Line => "line",
            Variant::Block => "block",
        })
    }
}

impl Variant {
    pub const fn start(&self) -> &'static str {
        match self {
            Variant::Line => LINE_START,
            Variant::Block => BLOCK_START,
        }
    }

    pub const fn end(&self) -> &'static str {
        match self {
            Variant::Line => LINE_END,
            Variant::Block => BLOCK_END,
        }
    }
}
