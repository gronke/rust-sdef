//! Negative-path coverage for [`sdef::Dictionary`].
//!
//! Each test pins a behaviour on malformed, incomplete, or unusual input.
//! The happy-path suite lives in `tests/parses.rs`; this file exists so
//! regressions on the *error* surface (and on edge cases that should
//! succeed but rarely appear in real fixtures) are caught explicitly.
//!
//! Where possible we assert on the *typed error variant* rather than the
//! message — quick-xml's wording is not a stable contract.

use std::io::Cursor;

use sdef::{CocoaBooleanValue, Dictionary, Error};

// ============================================================================
// Required-attribute handling
// ============================================================================

/// `<suite name="…">` is declared with `code #REQUIRED` in the DTD. quick-xml
/// surfaces missing required fields as a deserialization error rather than
/// silently defaulting. Pin that contract: a future switch to permissive
/// deserialisation would mask schema-violating sdefs.
#[test]
fn missing_required_attribute_surfaces_typed_error() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="missing-code"/>
        </dictionary>"#;

    let err = xml
        .parse::<Dictionary>()
        .expect_err("missing required `code` on <suite> must error");
    assert!(
        matches!(err, Error::Xml(_)),
        "expected Error::Xml from missing #REQUIRED attr, got {err:?}"
    );
}

// ============================================================================
// Empty and minimal documents
// ============================================================================

/// `<dictionary></dictionary>` with no suites is a valid sdef shape per the
/// DTD (the DTD permits zero or more suites). Parsing must succeed; the
/// suites vector must be empty.
#[test]
fn empty_dictionary_parses_with_zero_suites() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary></dictionary>"#;
    let dict: Dictionary = xml.parse().expect("empty dictionary must parse");
    assert!(dict.suites.is_empty());
}

// ============================================================================
// Encoding edge cases
// ============================================================================

/// Invalid UTF-8 bytes in the document body must produce a typed XML or I/O
/// error, never a panic. quick-xml's reader rejects non-UTF-8 input at the
/// byte level.
#[test]
fn invalid_utf8_input_returns_err() {
    // Lone continuation byte — not valid UTF-8.
    let bytes: &[u8] = &[0x80, 0x81, 0x82];
    let err = Dictionary::from_reader(Cursor::new(bytes))
        .expect_err("invalid UTF-8 must surface as a typed error");
    assert!(
        matches!(err, Error::Io(_) | Error::Xml(_)),
        "expected Error::Io or Error::Xml from invalid UTF-8, got {err:?}"
    );
}

// ============================================================================
// Closed-enum case sensitivity (DTD requires uppercase YES/NO)
// ============================================================================

/// `<cocoa boolean-value="yes">` (lowercase) is outside the DTD's closed
/// enum `(YES|NO)`. Strict mode must reject it with the typed
/// `UnknownAttributeValue` variant.
#[test]
fn lowercase_cocoa_boolean_value_strict_rejects() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <class name="thing" code="thng">
                    <property name="p" code="prop" type="boolean">
                        <cocoa boolean-value="yes"/>
                    </property>
                </class>
            </suite>
        </dictionary>"#;
    let err = Dictionary::from_str_strict(xml)
        .expect_err("lowercase boolean-value must be rejected in strict mode");
    match err {
        Error::UnknownAttributeValue {
            element,
            attribute,
            value,
        } => {
            assert_eq!(element, "cocoa");
            assert_eq!(attribute, "boolean-value");
            assert_eq!(value, "yes");
        }
        other => panic!("expected UnknownAttributeValue, got {other:?}"),
    }
}

/// Same input in lenient mode: the value is preserved verbatim as
/// `CocoaBooleanValue::Other(...)`. Pins the forward-compatibility escape
/// hatch behaviour.
#[test]
fn lowercase_cocoa_boolean_value_lenient_keeps_other() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <class name="thing" code="thng">
                    <property name="p" code="prop" type="boolean">
                        <cocoa boolean-value="yes"/>
                    </property>
                </class>
            </suite>
        </dictionary>"#;
    let dict: Dictionary = xml.parse().expect("lenient must accept");
    let prop = &dict.suites[0].classes[0].properties[0];
    let cocoa = prop.cocoa.as_ref().expect("property has <cocoa>");
    assert_eq!(
        cocoa.boolean_value,
        Some(CocoaBooleanValue::Other("yes".to_owned()))
    );
}

// ============================================================================
// Root-level documentation siblings
// ============================================================================

/// The DTD permits `<documentation>` as a direct child of `<dictionary>`
/// (sibling to `<suite>`), not only inside `<suite>` and `<command>`. Most
/// real fixtures only carry suite-level documentation; pin that the
/// dictionary-level form survives parsing in lenient mode.
///
/// We assert on `dict.suites` instead of `dict.documentation` because the
/// dictionary-root documentation field is not currently exposed on the AST
/// — confirming that parsing tolerates the construct (silently drops it,
/// per quick-xml's default) is the contract being pinned here.
#[test]
fn multiple_root_documentation_siblings_tolerated_in_lenient_mode() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <documentation><html>One.</html></documentation>
            <documentation><html>Two.</html></documentation>
            <documentation><html>Three.</html></documentation>
            <suite name="S" code="SUIT">
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;
    let dict: Dictionary = xml
        .parse()
        .expect("lenient must tolerate dictionary-level documentation");
    assert_eq!(dict.suites.len(), 1);
    assert_eq!(dict.suites[0].commands[0].name, "c");
}

// ============================================================================
// `<html>` content escaping
// ============================================================================

/// `<html>` content with entity-encoded markup must parse, and the entities
/// must be expanded by the reader so the AST holds the decoded text. This
/// pins the "we don't preserve raw bytes" contract — consumers can rely on
/// the `html: Vec<String>` field carrying decoded characters.
#[test]
fn html_with_entity_encoded_markup_parses_to_decoded_text() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <documentation>
                    <html>&lt;b&gt;bold&lt;/b&gt; &amp; italic</html>
                </documentation>
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;
    let dict: Dictionary = xml.parse().expect("entity-encoded html must parse");
    let docs = &dict.suites[0].documentation;
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].html.len(), 1);
    let body = &docs[0].html[0];
    assert!(
        body.contains("<b>") && body.contains("</b>") && body.contains("&"),
        "expected entity-decoded markup, got {body:?}"
    );
}

/// `<html>` wrapped in CDATA also parses; the CDATA wrapper is unwrapped
/// and the verbatim bytes appear in `html[0]`.
#[test]
fn html_with_cdata_section_parses() {
    let xml = "<?xml version=\"1.0\"?>\n\
        <dictionary>\n\
            <suite name=\"S\" code=\"SUIT\">\n\
                <documentation>\n\
                    <html><![CDATA[<b>bold</b> & italic]]></html>\n\
                </documentation>\n\
                <command name=\"c\" code=\"SUITcmd1\"/>\n\
            </suite>\n\
        </dictionary>";
    let dict: Dictionary = xml.parse().expect("cdata html must parse");
    let body = &dict.suites[0].documentation[0].html[0];
    assert!(
        body.contains("<b>") && body.contains("</b>"),
        "expected CDATA contents intact, got {body:?}"
    );
}

// ============================================================================
// Filesystem entry points
// ============================================================================

/// `Dictionary::from_path` on a non-existent path must surface the I/O
/// error as the typed `Error::Io` variant, not as `Error::Xml` or a panic.
#[test]
fn from_path_nonexistent_returns_io_error() {
    let err = Dictionary::from_path("/this/path/intentionally/does/not/exist.sdef")
        .expect_err("non-existent path must error");
    assert!(
        matches!(err, Error::Io(_)),
        "expected Error::Io for missing file, got {err:?}"
    );
}

/// Strict-mode counterpart: `from_path_strict` must also surface a missing
/// file as `Error::Io`, *not* as a strict-validation error (the I/O failure
/// happens before any XML is read).
#[test]
fn from_path_strict_nonexistent_returns_io_error() {
    let err = Dictionary::from_path_strict("/this/path/intentionally/does/not/exist.sdef")
        .expect_err("non-existent path must error");
    assert!(
        matches!(err, Error::Io(_)),
        "expected Error::Io for missing file, got {err:?}"
    );
}
