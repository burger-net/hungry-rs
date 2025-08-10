use std::ops::{ControlFlow, Range};

use crate::read::comments::{Comment, Comments, Either, Variant};

impl<'a> Comments<'a> {
    fn skip_content(&mut self) -> ControlFlow<Range<usize>, Variant> {
        // Return previously found comment variant.
        if let Some(variant) = self.var.take() {
            return ControlFlow::Continue(variant);
        }

        // Return remaining content if comment is not found.
        let Some((variant, index)) = Comments::find_start(&self.buf[self.pos..]) else {
            let start = self.pos;

            self.pos = self.buf.len();

            return ControlFlow::Break(start..self.pos);
        };

        // Return content before the comment.
        if index > 0 {
            self.var = Some(variant);

            let start = self.pos;

            self.pos += index;

            return ControlFlow::Break(start..self.pos);
        }

        ControlFlow::Continue(variant)
    }

    fn find_start(content: &str) -> Option<(Variant, usize)> {
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

    fn find_end(&mut self, variant: &Variant) -> (Range<usize>, bool) {
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

        let variant = match self.skip_content() {
            ControlFlow::Continue(variant) => variant,
            ControlFlow::Break(range) => {
                return Some(Either::Content { range });
            }
        };

        self.pos += variant.start().len();

        let (range, ended) = self.find_end(&variant);

        Some(Either::Comment {
            comment: Comment { variant, range },
            ended,
        })
    }
}
