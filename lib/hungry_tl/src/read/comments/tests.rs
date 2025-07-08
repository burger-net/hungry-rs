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
    let expected = &format!("{S1} {LINE_END} {S2} {BLOCK_END} {S3}");

    assert_eq!(collect(expected), [content(0, expected)])
}

#[test]
fn test_complex() {
    let s = &format!(" {S1} ");
    let l = &format!(" {S2} {BLOCK_START} {S3} {BLOCK_END} {S4} ");
    let m = &format!(" {S5} ");
    let b = &format!(" {S6} {LINE_START} {S7} {LINE_END} {S8} ");
    let e = &format!(" {S9} ");

    let result = &format!("{s}{LINE_START}{l}{LINE_END}{m}{BLOCK_START}{b}{BLOCK_END}{e}");

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

    assert_eq!(collect(result), expected)
}

#[test]
fn test_unended() {
    let result = &format!("{S1}{LINE_START}{S2}");

    let expected = [
        content(0, S1),
        comment(S1.len() + LINE_START.len(), S2, Variant::Line, false),
    ];

    assert_eq!(collect(result), expected);

    let result = &format!("{S3}{BLOCK_START}{S4}");

    let expected = [
        content(0, S3),
        comment(S3.len() + BLOCK_START.len(), S4, Variant::Block, false),
    ];

    assert_eq!(collect(result), expected);
}
