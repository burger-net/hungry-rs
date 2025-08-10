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
    let expected = &format!(" / / * * / {A} \n {B} */ {C} ");

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
    let s = &format!(" {A} ");
    let l = &format!(" {B} {BLOCK_START} {C} {BLOCK_END} {D} ");
    let m = &format!(" {E} ");
    let b = &format!(" {F} {LINE_START} {G} {LINE_END} {H} ");
    let e = &format!(" {I} ");

    let mut offset = Offset(0);

    let expected = [
        offset.content(s),
        offset.comment(l, Variant::Line, true),
        offset.content(m),
        offset.comment(b, Variant::Block, true),
        offset.content(e),
    ];

    assert_eq!(
        collect(&format!(
            "{s}{LINE_START}{l}{LINE_END}{m}{BLOCK_START}{b}{BLOCK_END}{e}"
        )),
        expected
    )
}
