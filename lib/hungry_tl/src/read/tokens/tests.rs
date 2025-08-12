use super::*;

use TokenKind::*;

use crate::read::comments::{Comment, Variant};

macro_rules! token {
    (Unknown in $range:expr) => {
        Err(UnknownToken($range))
    };

    (Literal in $range:expr) => {
        Ok(Token {
            kind: Literal {
                len: $range.end - $range.start,
            },
            pos: $range.start,
        })
    };

    ($kind:ident in $pos:expr) => {
        Ok(Token {
            kind: $kind,
            pos: $pos,
        })
    };
}

macro_rules! test {
    [ $func:ident => $buf:literal; $( $kind:ident in $span:expr ),* $(,)? ] => {
        #[test]
        fn $func() {
            assert_eq!(
                Tokens::new($buf).collect::<Vec<_>>(),
                [ $( token!($kind in $span), )* ],
            );
        }
    };
}

test![
    test_kinds => "vector#1cb5c415 {t:Type} # [ t ] = Vector t;";

    Literal in 0..6,
    Hash in 6,
    Literal in 7..15,
    OpenBrace in 16,
    Literal in 17..18,
    Colon in 18,
    Literal in 19..23,
    CloseBrace in 23,
    Hash in 25,
    OpenBracket in 27,
    Literal in 29..30,
    CloseBracket in 31,
    Equals in 33,
    Literal in 35..41,
    Literal in 42..43,
    Semicolon in 43,
];

test![
    test_triple_minus => "A`-B----+--C   бургерD~! ------  -- --- - E";

    Literal in 0..1,
    Unknown in 1..2,
    Unknown in 2..3,
    Literal in 3..4,
    TripleMinus in 4,
    Unknown in 7..8,
    Plus in 8,
    Unknown in 9..10,
    Unknown in 10..11,
    Literal in 11..12,
    Unknown in 15..27,
    Literal in 27..28,
    Unknown in 28..29,
    ExclamationMark in 29,
    TripleMinus in 31,
    TripleMinus in 34,
    Unknown in 39..40,
    Unknown in 40..41,
    TripleMinus in 42,
    Unknown in 46..47,
    Literal in 48..49,
];

#[test]
fn test_comment_collection() {
    let mut tokens = Tokens::new(
        "//@description Error.\n\
        //@code Error code\n\
        //@text Message\n\
        error#c4b9f9bb code:int text:string = Error;",
    );

    while let Some(item) = tokens.next() {
        item.unwrap();
    }

    let expected = [2..21, 24..40, 43..56].map(|range| Comment {
        variant: Variant::Line,
        range,
    });

    assert_eq!(tokens.comments(), expected);
}
