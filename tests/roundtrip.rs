//! Round-trip identity tests for [`sdef::Dictionary`].
//!
//! Invariant: `parse(emit(parse(input))) == parse(input)`. We do *not*
//! assert byte-level identity against the original document — XML has many
//! syntactically equivalent forms (attribute order, self-closing tags,
//! whitespace) and quick-xml does not preserve them. AST-level identity is
//! the actually-useful guarantee.

use sdef::{Access, AccessorStyle, CocoaBooleanValue, Dictionary};

fn assert_roundtrip(path: &str) {
    let original = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let ast1: Dictionary = original
        .parse()
        .unwrap_or_else(|e| panic!("first parse of {path}: {e}"));
    let emitted = ast1
        .to_xml_string()
        .unwrap_or_else(|e| panic!("serialize {path}: {e}"));
    let ast2: Dictionary = emitted
        .parse()
        .unwrap_or_else(|e| panic!("reparse of emitted {path}: {e}\nemitted XML:\n{emitted}"));
    assert_eq!(
        ast1, ast2,
        "AST identity must hold across parse->emit->parse for {path}"
    );
}

#[test]
fn roundtrip_mini() {
    assert_roundtrip("tests/fixtures/mini.sdef");
}

#[test]
fn roundtrip_synthetic() {
    assert_roundtrip("tests/fixtures/synthetic.sdef");
}

#[test]
fn roundtrip_extras() {
    assert_roundtrip("tests/fixtures/extras.sdef");
}

/// Strict-mode parse should accept the synthetic + extras fixtures
/// post-roundtrip. This is a stronger guarantee than basic AST identity:
/// the emitted output must contain only DTD-modelled elements and
/// in-range closed-enum values.
#[test]
fn roundtrip_passes_strict_mode() {
    for path in [
        "tests/fixtures/mini.sdef",
        "tests/fixtures/synthetic.sdef",
        "tests/fixtures/extras.sdef",
    ] {
        let original = std::fs::read_to_string(path).expect(path);
        let ast: Dictionary = original.parse().expect(path);
        let emitted = ast.to_xml_string().expect(path);
        Dictionary::from_str_strict(&emitted)
            .unwrap_or_else(|e| panic!("strict parse of emitted {path}: {e}\n{emitted}"));
    }
}

/// `AccessorStyle::Other("custom")` must round-trip verbatim. The strict
/// path would reject it, but the lenient path preserves it for forward
/// compatibility — and serialise must respect that.
#[test]
fn accessor_style_other_roundtrips_verbatim() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <class name="thing" code="thng">
                    <element type="bit">
                        <accessor style="hyper-relative"/>
                    </element>
                </class>
            </suite>
        </dictionary>"#;
    let ast1: Dictionary = xml.parse().expect("lenient parse");
    let accessor = &ast1.suites[0].classes[0].elements[0].accessors[0];
    assert_eq!(
        accessor.style,
        AccessorStyle::Other("hyper-relative".to_owned())
    );
    let emitted = ast1.to_xml_string().expect("serialize");
    assert!(
        emitted.contains(r#"style="hyper-relative""#),
        "Other value should be emitted verbatim, got {emitted}"
    );
    let ast2: Dictionary = emitted.parse().expect("reparse");
    assert_eq!(ast1, ast2);
}

/// `<documentation>` with multiple `<html>` children must preserve the
/// children, their order, and their text content across a round-trip.
#[test]
fn documentation_with_multiple_html_children_roundtrips() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <documentation>
                    <html>First snippet.</html>
                    <html>Second snippet.</html>
                    <html>Third snippet.</html>
                </documentation>
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;
    let ast1: Dictionary = xml.parse().expect("lenient parse");
    assert_eq!(
        ast1.suites[0].documentation[0].html,
        vec![
            "First snippet.".to_owned(),
            "Second snippet.".to_owned(),
            "Third snippet.".to_owned(),
        ]
    );
    let emitted = ast1.to_xml_string().expect("serialize");
    let ast2: Dictionary = emitted.parse().expect("reparse");
    assert_eq!(ast1, ast2);
}

/// Interleaved children at the suite level (`<command>` / `<class>` /
/// `<class-extension>` mixed in arbitrary order) must AST-equal after
/// round-trip even though the emitter groups them by Vec on the way out.
#[test]
fn suite_interleaved_children_ast_identity() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <command name="a" code="SUITcmda"/>
                <class name="cls1" code="cls1"/>
                <command name="b" code="SUITcmdb"/>
                <class name="cls2" code="cls2"/>
                <command name="c" code="SUITcmdc"/>
            </suite>
        </dictionary>"#;
    let ast1: Dictionary = xml.parse().expect("lenient parse");
    let cmd_names: Vec<&str> = ast1.suites[0]
        .commands
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    let class_names: Vec<&str> = ast1.suites[0]
        .classes
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(cmd_names, vec!["a", "b", "c"]);
    assert_eq!(class_names, vec!["cls1", "cls2"]);
    let emitted = ast1.to_xml_string().expect("serialize");
    let ast2: Dictionary = emitted.parse().expect("reparse");
    assert_eq!(ast1, ast2);
}

/// Typed `Access` and `CocoaBooleanValue` enums on emitted XML must use
/// the canonical DTD spellings.
#[test]
fn typed_enums_emit_canonical_strings() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <class name="thing" code="thng">
                    <property name="p" code="prop" type="text" access="r">
                        <cocoa boolean-value="YES"/>
                    </property>
                </class>
            </suite>
        </dictionary>"#;
    let ast: Dictionary = xml.parse().expect("parse");
    let prop = &ast.suites[0].classes[0].properties[0];
    assert_eq!(prop.access, Some(Access::Read));
    assert_eq!(
        prop.cocoa.as_ref().unwrap().boolean_value,
        Some(CocoaBooleanValue::Yes)
    );
    let emitted = ast.to_xml_string().expect("serialize");
    assert!(emitted.contains(r#"access="r""#), "got {emitted}");
    assert!(emitted.contains(r#"boolean-value="YES""#), "got {emitted}");
}
