#[cfg(test)]
mod tests;

use std::{cmp, fmt};

const LINE_START: &str = "//";
const LINE_END: &str = "\n";
const BLOCK_START: &str = "/*";
const BLOCK_END: &str = "*/";

pub(super) struct Comments<'a> {
    buf: &'a str,
    pos: usize,
}

impl<'a> Comments<'a> {
    pub(super) fn new(buf: &'a str) -> Self {
        Self { buf, pos: 0 }
    }

    pub(super) fn pos(&self) -> usize {
        self.pos
    }

    fn find_comment_start(content: &str) -> Option<(Variant, usize)> {
        match (content.find(LINE_START), content.find(BLOCK_START)) {
            (None, None) => None,
            (Some(line), None) => Some((Variant::Line, line)),
            (None, Some(block)) => Some((Variant::Block, block)),
            (Some(line), Some(block)) => match line.cmp(&block) {
                cmp::Ordering::Less => Some((Variant::Line, line)),
                cmp::Ordering::Equal => unreachable!("comment start patterns must differ"),
                cmp::Ordering::Greater => Some((Variant::Block, block)),
            },
        }
    }

    fn find_comment_end(&mut self, variant: &Variant) -> (usize, usize, bool) {
        let offset = self.pos;

        let pattern = variant.end();

        if let Some(length) = self.buf[self.pos..].find(pattern) {
            self.pos += length + pattern.len();

            (offset, offset + length, true)
        } else {
            self.pos = self.buf.len();

            (offset, self.pos, false)
        }
    }
}

impl<'a> Iterator for Comments<'a> {
    type Item = Either<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(
            self.pos <= self.buf.len(),
            "position exceeds buffer's length"
        );

        if self.pos >= self.buf.len() {
            return None;
        }

        let Some((variant, start)) = Self::find_comment_start(&self.buf[self.pos..]) else {
            // No comments found. Return remaining content.
            let offset = self.pos;
            self.pos = self.buf.len();

            return Some(Either::Content {
                content: &self.buf[offset..],
                offset,
            });
        };

        // Return content before the comment, if any.
        if start > 0 {
            let offset = self.pos;
            self.pos += start;

            return Some(Either::Content {
                content: &self.buf[offset..self.pos],
                offset,
            });
        }

        self.pos += variant.start().len();

        let (offset, end, ended) = self.find_comment_end(&variant);

        Some(Either::Comment {
            comment: Comment {
                variant,
                content: &self.buf[offset..end],
            },
            offset,
            ended,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Either<'a> {
    Comment {
        comment: Comment<'a>,
        offset: usize,
        ended: bool,
    },
    Content {
        content: &'a str,
        offset: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment<'a> {
    pub variant: Variant,
    pub content: &'a str,
}

impl<'a> fmt::Display for Comment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} comment ({:?})", self.variant, self.content)
    }
}

impl<'a> Comment<'a> {
    pub fn trim(&mut self) {
        self.content = self.content.trim()
    }

    pub fn trimmed(mut self) -> Self {
        self.trim();
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Variant {
    Line,
    Block,
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variant::Line => "line",
            Variant::Block => "block",
        }
        .fmt(f)
    }
}

impl Variant {
    pub const fn start(&self) -> &'static str {
        match self {
            Variant::Line => BLOCK_START,
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
