use std::fmt;

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

        // Find next comment index.
        let line = self.buf[self.pos..].find(LINE_START).map(Start::line);
        let block = self.buf[self.pos..].find(BLOCK_START).map(Start::block);

        let Some(start) = [line, block].into_iter().min_by(Start::min_cmp).unwrap() else {
            // No comments found. Return remaining content.

            let offset = self.pos;
            self.pos = self.buf.len();

            return Some(Either::Content {
                content: &self.buf[offset..],
                offset,
            });
        };

        // Return content before the comment, if any.
        if start.offset > 0 {
            let offset = self.pos;
            self.pos += start.offset;

            return Some(Either::Content {
                content: &self.buf[offset..self.pos],
                offset,
            });
        }

        self.pos += start.variant.start().len();

        let offset = self.pos;

        // Find comment end.
        let (end, ended) = if let Some(length) = self.buf[self.pos..].find(start.variant.stop()) {
            self.pos += length + start.variant.stop().len();

            (offset + length, true)
        } else {
            self.pos = self.buf.len();

            (self.pos, false)
        };

        Some(Either::Comment {
            comment: Comment {
                variant: start.variant,
                content: &self.buf[offset..end],
            },
            offset,
            ended,
        })
    }
}

struct Start {
    variant: Variant,
    offset: usize,
}

impl Start {
    // PartialOrd and Ord cannot be implemented for Option.
    fn min_cmp(l: &Option<Self>, r: &Option<Self>) -> std::cmp::Ordering {
        match (l, r) {
            (None, None) => std::cmp::Ordering::Equal,

            // NOTE: Reversed intentionally to select Some when one is present.
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,

            (Some(Start { offset: l, .. }), Some(Start { offset: r, .. })) => {
                debug_assert_ne!(l, r, "comment start patterns must differ");

                l.cmp(&r)
            }
        }
    }

    fn line(offset: usize) -> Self {
        Self {
            variant: Variant::Line,
            offset,
        }
    }

    fn block(offset: usize) -> Self {
        Self {
            variant: Variant::Block,
            offset,
        }
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

impl<'a> Comment<'a> {
    pub fn trim(&mut self) {
        self.content = self.content.trim()
    }

    pub fn trimmed(mut self) -> Self {
        self.trim();
        self
    }
}

impl<'a> fmt::Display for Comment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} comment ({:?})", self.variant, self.content)
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

    pub const fn stop(&self) -> &'static str {
        match self {
            Variant::Line => LINE_END,
            Variant::Block => BLOCK_END,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "SAMPLE";

    const S1: &str = "THE";
    const S2: &str = "QUICK";
    const S3: &str = "BROWN";
    const S4: &str = "FOX";
    const S5: &str = "JUMPS";
    const S6: &str = "OVER";
    const S7: &str = "THE";
    const S8: &str = "LAZY";
    const S9: &str = "DOG";

    fn collect(buf: &str) -> Vec<Either> {
        Comments::new(buf).collect()
    }

    fn comment(offset: usize, content: &str, variant: Variant, ended: bool) -> Either {
        Either::Comment {
            comment: Comment { variant, content },
            offset,
            ended,
        }
    }

    fn content(offset: usize, content: &str) -> Either {
        Either::Content { content, offset }
    }

    #[test]
    fn test_no_comments() {
        assert_eq!(collect(SAMPLE), [content(0, SAMPLE)]);
    }

    #[test]
    fn test_start_only() {
        assert_eq!(
            collect(&format!("{LINE_START}")),
            [comment(LINE_START.len(), "", Variant::Line, false)]
        );

        assert_eq!(
            collect(&format!("{BLOCK_START}")),
            [comment(BLOCK_START.len(), "", Variant::Block, false)]
        );
    }

    #[test]
    fn test_immediate() {
        assert_eq!(
            collect(&format!("{LINE_START}{SAMPLE}")),
            [comment(LINE_START.len(), SAMPLE, Variant::Line, false)]
        );

        assert_eq!(
            collect(&format!("{BLOCK_START}{SAMPLE}")),
            [comment(BLOCK_START.len(), SAMPLE, Variant::Block, false)]
        );
    }

    #[test]
    fn test_unstarted() {
        let buffer = &format!("{S1} {LINE_END} {S2} {BLOCK_END} {S3}");

        assert_eq!(collect(buffer), [content(0, buffer)])
    }

    #[test]
    fn test_complex() {
        let s = &format!(" {S1} ");
        let l = &format!(" {S2} {BLOCK_START} {S3} {BLOCK_END} {S4} ");
        let m = &format!(" {S5} ");
        let b = &format!(" {S6} {LINE_START} {S7} {LINE_END} {S8} ");
        let e = &format!(" {S9} ");

        let buf = &format!("{s}{LINE_START}{l}{LINE_END}{m}{BLOCK_START}{b}{BLOCK_END}{e}");

        let mut offset = 0;
        let mut offset = |s1: &str, s2: &str| {
            offset += s1.len() + s2.len();
            offset
        };

        let expected = [
            content(0, s),
            comment(offset(s, LINE_START), l, Variant::Line, true),
            content(offset(l, LINE_END), m),
            comment(offset(m, BLOCK_START), b, Variant::Block, true),
            content(offset(b, BLOCK_END), e),
        ];

        assert_eq!(collect(buf), expected)
    }

    #[test]
    fn test_unended() {
        let expected = [
            content(0, S1),
            comment(S1.len() + LINE_START.len(), S2, Variant::Line, false),
        ];

        assert_eq!(collect(&format!("{S1}{LINE_START}{S2}")), expected);

        let expected = [
            content(0, S3),
            comment(S3.len() + BLOCK_START.len(), S4, Variant::Block, false),
        ];

        assert_eq!(collect(&format!("{S3}{BLOCK_START}{S4}")), expected);
    }
}
