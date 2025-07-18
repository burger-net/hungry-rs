use std::fmt;
use std::ops::{ControlFlow, Range};

pub const LINE_START: &str = "//";
pub const LINE_END: &str = "\n";
pub const BLOCK_START: &str = "/*";
pub const BLOCK_END: &str = "*/";

pub(super) struct Comments<'a> {
    buf: &'a str,
    pos: usize,
    cur: Option<Variant>,
}

impl<'a> Comments<'a> {
    pub(super) fn new(buf: &'a str) -> Self {
        Self {
            buf,
            pos: 0,
            cur: None,
        }
    }

    pub(super) fn buf(&self) -> &'a str {
        self.buf
    }

    pub(super) fn pos(&self) -> usize {
        self.pos
    }

    fn collect_content(&mut self) -> ControlFlow<Range<usize>, Variant> {
        // Return previously found comment variant.
        if let Some(variant) = self.cur.take() {
            return ControlFlow::Continue(variant);
        }

        // Return remaining content if comment is not found.
        let Some((variant, index)) = Comments::find_comment_start(&self.buf[self.pos..]) else {
            let start = self.pos;

            self.pos = self.buf.len();

            return ControlFlow::Break(start..self.pos);
        };

        // Return content before the comment.
        if index > 0 {
            self.cur = Some(variant);

            let start = self.pos;

            self.pos += index;

            return ControlFlow::Break(start..self.pos);
        }

        ControlFlow::Continue(variant)
    }

    fn find_comment_start(content: &str) -> Option<(Variant, usize)> {
        let mut chars = content.char_indices();

        loop {
            let (index, '/') = chars.next()? else {
                continue;
            };

            let variant = match chars.next()?.1 {
                '/' => Variant::Line,
                '*' => Variant::Block,
                _ => continue,
            };

            return Some((variant, index));
        }
    }

    fn find_comment_end(&mut self, variant: &Variant) -> (Range<usize>, bool) {
        let start = self.pos;

        let pattern = variant.end();

        if let Some(index) = self.buf[self.pos..].find(pattern) {
            self.pos += index + pattern.len();

            (start..start + index, true)
        } else {
            self.pos = self.buf.len();

            (start..self.pos, false)
        }
    }
}

impl<'a> Iterator for Comments<'a> {
    type Item = Either;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.pos <= self.buf.len());

        if self.pos >= self.buf.len() {
            return None;
        }

        let variant = match self.collect_content() {
            ControlFlow::Continue(variant) => variant,
            ControlFlow::Break(range) => {
                return Some(Either::Content { range });
            }
        };

        self.pos += variant.start().len();

        let (range, ended) = self.find_comment_end(&variant);

        Some(Either::Comment {
            comment: Comment { variant, range },
            ended,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "SAMPLE";

    const A: &str = "THE";
    const B: &str = "QUICK";
    const C: &str = "BROWN";
    const D: &str = "FOX";
    const E: &str = "JUMPS";
    const F: &str = "OVER";
    const G: &str = "THE";
    const H: &str = "LAZY";
    const I: &str = "DOG";

    fn collect(buf: &str) -> Vec<Either> {
        Comments::new(buf).collect()
    }

    struct Offset(usize);

    impl Offset {
        fn content(&mut self, content: &str) -> Either {
            let start = self.0;
            self.0 += content.len();

            Either::Content {
                range: start..self.0,
            }
        }

        fn comment(&mut self, content: &str, variant: Variant, ended: bool) -> Either {
            let start = self.0 + variant.start().len();
            let end = start + content.len();
            self.0 = end + variant.end().len() * ended as usize;

            Either::Comment {
                comment: Comment {
                    variant,
                    range: start..end,
                },
                ended,
            }
        }
    }

    #[test]
    fn test_absence() {
        let expected = &format!(" / / * * / {A} {LINE_END} {B} {BLOCK_END} {C} ");

        assert_eq!(collect(expected), [Offset(0).content(expected)]);
    }

    #[test]
    fn test_immediate_unended() {
        assert_eq!(
            collect(&format!("{LINE_START}{SAMPLE}")),
            [Offset(0).comment(SAMPLE, Variant::Line, false)]
        );

        assert_eq!(
            collect(&format!("{BLOCK_START}{SAMPLE}")),
            [Offset(0).comment(SAMPLE, Variant::Block, false)]
        );
    }

    #[test]
    fn test_consecutive_and_empty() {
        let mut offset = Offset(0);

        let expected = [
            offset.content(D),
            offset.comment(E, Variant::Line, true),
            offset.comment(F, Variant::Block, true),
            offset.content(G),
            offset.comment("", Variant::Line, true),
            offset.content(H),
            offset.comment("", Variant::Block, true),
            offset.content(I),
        ];

        assert_eq!(
            collect(&format!(
                "{D}{LINE_START}{E}{LINE_END}{BLOCK_START}{F}{BLOCK_END}{G}{LINE_START}{LINE_END}{H}{BLOCK_START}{BLOCK_END}{I}"
            )),
            expected
        )
    }

    #[test]
    fn test_variants_and_whitespaces() {
        let a = &format!(" {A} ");
        let b = &format!(" {B} {BLOCK_START} {C} {BLOCK_END} {D} ");
        let c = &format!(" {E} ");
        let d = &format!(" {F} {LINE_START} {G} {LINE_END} {H} ");
        let e = &format!(" {I} ");

        let mut offset = Offset(0);

        let expected = [
            offset.content(a),
            offset.comment(b, Variant::Line, true),
            offset.content(c),
            offset.comment(d, Variant::Block, true),
            offset.content(e),
        ];

        assert_eq!(
            collect(&format!(
                "{a}{LINE_START}{b}{LINE_END}{c}{BLOCK_START}{d}{BLOCK_END}{e}"
            )),
            expected
        )
    }
}
