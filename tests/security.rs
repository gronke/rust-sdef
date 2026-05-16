//! Pinned security invariants for [`sdef::Dictionary`] parsing.
//!
//! Each test asserts a behaviour we rely on for safety against hostile or
//! merely careless input. They exist so a future change to the parser
//! that quietly weakens one of these guarantees will fail CI and force a
//! deliberate code review.
//!
//! Properties pinned here:
//! 1. `<xi:include>` is **never** resolved. Strict mode rejects it; lenient
//!    mode silently drops the element. In both cases the `href` attribute
//!    is never opened, parsed, or even path-resolved.
//! 2. The DOCTYPE prelude's external `SYSTEM` reference is **never** fetched.
//!    Apple's sdef files routinely declare a DOCTYPE pointing at
//!    `/System/Library/DTDs/sdef.dtd` via `file://localhost/...` — we
//!    parse the prelude as opaque bytes regardless of whether that path
//!    exists on the host.
//! 3. External-entity declarations inside the DOCTYPE are **never**
//!    auto-fetched. Even when referenced from body content (the classic
//!    XXE vector), the referenced URL/path is not opened.
//! 4. Internal-entity expansion is **bounded**. A billion-laughs-style
//!    document does not blow up memory or take exponential time to parse.
//!
//! These tests describe the *contract*, not the *implementation*. Any of
//! "parse errors gracefully" or "parse succeeds with the entity dropped"
//! satisfies a test as long as the underlying file/path is not actually
//! opened.

use sdef::{Dictionary, Error};

// ============================================================================
// XInclude
// ============================================================================

/// Lenient parsing must drop `<xi:include>` silently and never attempt to
/// open the `href`. The `href` here points at a path that does not exist;
/// if the parser tried to resolve it, parsing would fail with a file
/// I/O error.
#[test]
fn lenient_drops_xi_include_without_following_href() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary xmlns:xi="http://www.w3.org/2001/XInclude">
            <suite name="probe" code="SUIT">
                <xi:include href="/this/path/intentionally/does/not/exist.sdef"/>
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;

    let dict: Dictionary = xml
        .parse()
        .expect("lenient parse must succeed without trying to open the include href");
    assert_eq!(dict.suites.len(), 1);
    assert_eq!(dict.suites[0].commands.len(), 1);
    assert_eq!(dict.suites[0].commands[0].name, "c");
}

/// Same as above but with a path-traversal `href`. Confirms we don't even
/// path-resolve, let alone open. If the parser were to canonicalise this
/// against any base path, it would touch `/etc/passwd` (Unix) or fail; we
/// must do neither.
#[test]
fn lenient_drops_xi_include_with_path_traversal_href() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary xmlns:xi="http://www.w3.org/2001/XInclude">
            <suite name="probe" code="SUIT">
                <xi:include href="../../../../../../etc/passwd"/>
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;

    let dict: Dictionary = xml
        .parse()
        .expect("lenient parse must not resolve traversal href");
    assert_eq!(dict.suites[0].commands.len(), 1);
}

/// Strict mode rejects `<xi:include>` with a typed `XIncludeUnsupported`
/// error, *not* with a generic I/O or XML error. This pins the contract
/// that the rejection comes from our pre-pass (the `href` was never
/// inspected) rather than from a downstream file-system attempt.
#[test]
fn strict_rejects_xi_include_with_traversal_href() {
    let xml = r#"<?xml version="1.0"?>
        <dictionary xmlns:xi="http://www.w3.org/2001/XInclude">
            <suite name="probe" code="SUIT">
                <xi:include href="../../../../../../etc/passwd"/>
                <command name="c" code="SUITcmd1"/>
            </suite>
        </dictionary>"#;

    let err = Dictionary::from_str_strict(xml).expect_err("strict mode must reject xi:include");
    assert!(
        matches!(err, Error::XIncludeUnsupported),
        "expected Error::XIncludeUnsupported, got {err:?}"
    );
}

// ============================================================================
// DOCTYPE external SYSTEM reference
// ============================================================================

/// A DOCTYPE pointing at a non-existent path must not block parsing. This
/// is the same shape Apple's sdefs use (`SYSTEM "file://..."`), so we
/// inherit the property at runtime: the prelude is opaque bytes, never a
/// file fetch.
#[test]
fn external_doctype_system_id_is_not_fetched() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE dictionary SYSTEM "/this/path/intentionally/does/not/exist.dtd">
<dictionary>
    <suite name="probe" code="SUIT">
        <command name="c" code="SUITcmd1"/>
    </suite>
</dictionary>"#;

    let dict: Dictionary = xml
        .parse()
        .expect("DOCTYPE SYSTEM reference must not be fetched");
    assert_eq!(dict.suites[0].commands[0].name, "c");
}

/// As above, but with the system identifier pointing at an HTTP URL. We
/// must not perform any network I/O during parse.
#[test]
fn external_doctype_http_system_id_is_not_fetched() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE dictionary SYSTEM "http://invalid.example.test/sdef.dtd">
<dictionary>
    <suite name="probe" code="SUIT">
        <command name="c" code="SUITcmd1"/>
    </suite>
</dictionary>"#;

    let dict: Dictionary = xml
        .parse()
        .expect("DOCTYPE HTTP SYSTEM reference must not be fetched");
    assert_eq!(dict.suites[0].commands[0].name, "c");
}

// ============================================================================
// External entity expansion (XXE)
// ============================================================================

/// An external-entity declaration that is never referenced must not be
/// auto-fetched. The SYSTEM URL here points at a non-existent path; if the
/// parser tried to materialise the entity at declaration time, parsing
/// would fail with an I/O error.
#[test]
fn external_entity_declaration_alone_does_not_fetch() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE dictionary [
    <!ENTITY xxe SYSTEM "/this/path/intentionally/does/not/exist">
]>
<dictionary>
    <suite name="probe" code="SUIT">
        <command name="c" code="SUITcmd1"/>
    </suite>
</dictionary>"#;

    let dict: Dictionary = xml
        .parse()
        .expect("declaring an external entity (without referencing it) must not fetch");
    assert_eq!(dict.suites[0].name, "probe");
}

/// The XXE attack: an external entity is *referenced* from body content,
/// expecting the parser to substitute the file contents in place. We
/// require that no such substitution happens — either the reference is
/// dropped/errors, or it remains as something that does not contain the
/// referenced path's contents.
///
/// The assertion is deliberately tolerant about *which* safe outcome
/// occurs (parse-error or unresolved-reference are both fine), but
/// strict about the unsafe one: the file at the SYSTEM URL must not have
/// been read.
#[test]
fn external_entity_reference_does_not_leak_system_path() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE dictionary [
    <!ENTITY xxe SYSTEM "/this/path/intentionally/does/not/exist">
]>
<dictionary>
    <suite name="x&xxe;y" code="SUIT">
        <command name="c" code="SUITcmd1"/>
    </suite>
</dictionary>"#;

    match xml.parse::<Dictionary>() {
        Ok(dict) => {
            // Success means quick-xml did not attempt the fetch (the SYSTEM
            // path does not exist; opening it would have errored). Confirm
            // additionally that the path string itself didn't leak into the
            // parsed value.
            let name = &dict.suites[0].name;
            assert!(
                !name.contains("intentionally/does/not/exist"),
                "external entity body must not appear in AST: name = {name:?}"
            );
        }
        Err(Error::Xml(_)) => {
            // Acceptable: quick-xml refused to resolve the custom entity.
            // The key property is that this is an XML-deserialisation
            // error, not an I/O error from trying to open the SYSTEM path.
        }
        Err(other) => panic!(
            "expected Ok or Error::Xml; got {other:?} \
             — an I/O error here would indicate the SYSTEM path was opened"
        ),
    }
}

// ============================================================================
// Internal entity expansion (billion laughs)
// ============================================================================

/// A billion-laughs-style document declares nested internal entities that
/// would expand to 10^N copies if naively substituted. Parsing must
/// either reject the document or leave the entities unresolved — it must
/// *not* materialise a 10^6-character string in memory.
///
/// The bomb here is bounded to four levels (10^4 expansion) so the test
/// fails fast if quick-xml is vulnerable rather than hanging.
#[test]
fn internal_entity_expansion_is_bounded() {
    let xml = r#"<?xml version="1.0"?>
<!DOCTYPE dictionary [
    <!ENTITY lol "lol">
    <!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
    <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
    <!ENTITY lol4 "&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;&lol3;">
]>
<dictionary>
    <suite name="x&lol4;y" code="SUIT">
        <command name="c" code="SUITcmd1"/>
    </suite>
</dictionary>"#;

    match xml.parse::<Dictionary>() {
        Ok(dict) => {
            let name = &dict.suites[0].name;
            // Full expansion would be ~30k chars. A safe parser keeps the
            // suite name to the literal "x...y" template (with an
            // unresolved or empty entity). 1KB is a generous ceiling that
            // any safe behaviour clears, and that vulnerable expansion
            // would blow past.
            assert!(
                name.len() < 1024,
                "internal-entity expansion was not bounded: \
                 suite name is {} chars",
                name.len()
            );
        }
        Err(Error::Xml(_)) => {
            // Acceptable: quick-xml refused to resolve the custom entities.
        }
        Err(other) => panic!("unexpected error variant: {other:?}"),
    }
}
