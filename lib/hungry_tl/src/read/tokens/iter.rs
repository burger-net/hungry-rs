use crate::read::comments::{Comment, Comments, Either};
use crate::read::tokens::{Token, UnknownToken};

pub(in crate::read) struct Tokens<'a> {
    iterator: Comments<'a>,
    position: usize,
    contents: &'a str,
    comments: Vec<Comment>,
}

impl<'a> Tokens<'a> {
    pub(in crate::read) fn new(buf: &'a str) -> Self {
        Self {
            iterator: Comments::new(buf),
            position: 0,
            contents: "",
            comments: Vec::new(),
        }
    }

    pub(in crate::read) fn buf(&self) -> &'a str {
        self.iterator.buf()
    }

    pub(in crate::read) fn comments(&mut self) -> Vec<Comment> {
        std::mem::take(&mut self.comments)
    }

    fn advance(&mut self, offset: usize) {
        debug_assert!(offset <= self.contents.len());

        let content = &self.contents[offset..];

        self.contents = content.trim_start();
        self.position += offset + content.len() - self.contents.len();
    }

    fn content(&mut self) -> Option<&'a str> {
        if !self.contents.is_empty() {
            return Some(self.contents);
        }

        loop {
            match self.iterator.next()? {
                Either::Content { range } => {
                    let content = &self.buf()[range.clone()];
                    self.contents = content.trim_start();

                    self.position = range.start + content.len() - self.contents.len();

                    if !self.contents.is_empty() {
                        return Some(self.contents);
                    }
                }
                Either::Comment { comment, ended } => {
                    self.position = comment.range.end;

                    if ended {
                        self.position += comment.variant.end().len();
                    }

                    self.comments.push(comment);
                }
            }
        }
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token, UnknownToken>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.content()?.char_indices();

        let result = Token::parse(&mut chars, self.position);

        self.advance(match &result {
            Ok(Token { kind, .. }) => kind.len(),
            Err(UnknownToken(range)) => range.len(),
        });

        Some(result)
    }
}
