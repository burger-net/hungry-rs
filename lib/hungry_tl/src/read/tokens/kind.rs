use std::fmt;
use std::str::CharIndices;

use crate::read::tokens::{Token, UnknownToken};

impl TokenKind {
    fn literal(chars: &mut CharIndices) -> TokenKind {
        let len = loop {
            let Some((i, c)) = chars.next() else {
                break chars.offset();
            };

            if !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_') {
                break i;
            };
        };

        TokenKind::Literal { len }
    }

    fn unknown(chars: &mut CharIndices, pos: usize) -> UnknownToken {
        let len = loop {
            let Some((i, c)) = chars.next() else {
                break chars.offset();
            };

            if matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' |
                ':' | ';' | '(' | ')' | '[' | ']' | '_' |
                '{' | '}' | '=' | '#' | '?' | '%' | '+' |
                '<' | '>' | ',' | '-' | '.' | '!' | '*')
                || c.is_whitespace()
            {
                break i;
            }
        };

        UnknownToken(pos..pos + len)
    }
}

macro_rules! impl_kind {
    { $( $char:literal => $token:ident as $name:literal ),+ $(,)? } => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum TokenKind {
            Literal { len: usize },
            TripleMinus,
            $( $token, )+
        }

        impl fmt::Display for TokenKind {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    TokenKind::Literal { len } => return write!(f, "literal with length {len}"),
                    TokenKind::TripleMinus => "triple minus",
                    $( TokenKind::$token => $name, )+
                }.fmt(f)
            }
        }

        impl TokenKind {
            pub const TRIPLE_MINUS: &str = "---";

            pub const fn len(&self) -> usize {
                match self {
                    TokenKind::Literal { len } => *len,
                    TokenKind::TripleMinus => TokenKind::TRIPLE_MINUS.len(),
                    $( TokenKind::$token => $char.len_utf8(), )+
                }
            }
        }

        impl Token {
            /// Panics if `chars` is empty
            pub(super) fn parse(chars: &mut CharIndices, pos: usize) -> Result<Self, UnknownToken> {
                let kind = match chars.next().unwrap().1 {
                    $( $char => TokenKind::$token, )+

                    '-' if chars.as_str().starts_with("--") => TokenKind::TripleMinus,

                    'a'..='z' | 'A'..='Z' | '0'..='9' => TokenKind::literal(chars),

                    _ => return Err(TokenKind::unknown(chars, pos)),
                };

                Ok(Token { kind, pos })
            }
        }
    };
}

impl_kind! {
    ':' => Colon as "colon",
    ';' => Semicolon as "semicolon",
    '(' => OpenParenthesis as "open parenthesis",
    ')' => CloseParenthesis as "close parenthesis",
    '[' => OpenBracket as "open bracket",
    ']' => CloseBracket as "close bracket",
    '{' => OpenBrace as "open brace",
    '}' => CloseBrace as "close brace",
    '=' => Equals as "equals",
    '#' => Hash as "hash",
    '?' => QuestionMark as "question mark",
    '%' => Percent as "percent",
    '+' => Plus as "plus",
    '<' => OpenAngle as "open angle",
    '>' => CloseAngle as "close angle",
    ',' => Comma as "comma",
    '.' => Dot as "dot",
    '*' => Asterisk as "asterisk",
    '!' => ExclamationMark as "exclamation mark",
}
