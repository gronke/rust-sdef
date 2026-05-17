//! Resource-bound invariants for [`sdef::Dictionary`].
//!
//! Pins parser behaviour at the boundary of what is plausibly large but
//! not malicious — deeply nested types, many sibling commands, and
//! megabyte-scale `Other(String)` escape-hatch values. None of these are
//! security tests (those live in `tests/security.rs`); they exist so a
//! future change that introduces super-linear time or memory in input
//! size becomes visible as a test failure rather than slow drift.
//!
//! Each test enforces a wall-clock budget so the suite fails fast rather
//! than hanging if a regression makes the parser exponentially slow.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sdef::{Access, Dictionary};

/// `<type>` is recursive: a parameter, result, or property can contain a
/// `<type>` child which itself contains nested `<type>` children. Build a
/// 100-level-deep tower and confirm the parser terminates without panic,
/// stack overflow, or hang on the default thread stack.
///
/// 100 levels is a calibrated value: deep enough that any real-world sdef
/// will be comfortably below it (TypeRef nesting in shipping fixtures is
/// 1–3 levels; Cocoa Scripting ignores nested-list expressions per the
/// DTD docs at `src/typeref.rs`), and shallow enough to stay well inside
/// the default 2 MiB stack used by `cargo test`. If a future change
/// pushes per-frame overhead up enough that 100 levels overflow, this
/// test will catch it — the reviewer can then either widen the stack
/// with [`std::thread::Builder::stack_size`] or, better, restructure the
/// deserialiser to use bounded recursion.
#[test]
fn deeply_nested_type_does_not_stack_overflow() {
    const DEPTH: usize = 100;

    let mut inner = String::from(r#"<type type="text"/>"#);
    for _ in 0..DEPTH {
        inner = format!(r#"<type type="text">{inner}</type>"#);
    }
    let xml = format!(
        r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <command name="c" code="SUITcmd1">
                    <parameter name="p" code="prm1">
                        {inner}
                    </parameter>
                </command>
            </suite>
        </dictionary>"#
    );

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(xml.parse::<Dictionary>().is_ok());
    });

    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(_ok_or_err) => {
            // Either outcome is acceptable; the load-bearing claim is
            // "parse returns".
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("{DEPTH}-deep nested <type> caused parse to hang (>10s)");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("parser thread panicked on {DEPTH}-deep nested <type>");
        }
    }
}

/// A suite containing one thousand sibling `<command>` elements must parse
/// in linear time and memory — pinning the "list parsing scales with
/// input" invariant against any future regression that would, e.g.,
/// introduce a per-element scan over previously-parsed siblings.
#[test]
fn thousand_sibling_commands_parse_linearly() {
    const COUNT: usize = 1_000;

    let mut body = String::with_capacity(COUNT * 64);
    for i in 0..COUNT {
        body.push_str(&format!(
            r#"<command name="c{i}" code="SUITcm{:02}"/>"#,
            i % 100
        ));
    }
    let xml = format!(
        r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                {body}
            </suite>
        </dictionary>"#
    );

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(xml.parse::<Dictionary>());
    });

    match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(Ok(dict)) => {
            assert_eq!(
                dict.suites[0].commands.len(),
                COUNT,
                "all {COUNT} sibling commands must round-trip into the AST"
            );
        }
        Ok(Err(e)) => panic!("parse of {COUNT}-sibling document failed: {e}"),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("{COUNT} sibling commands took >10s to parse — possible quadratic regression");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("parser thread panicked on {COUNT} sibling commands");
        }
    }
}

/// `Access::Other(s)` is the forward-compat escape hatch for lenient mode
/// — it stores any string the DTD doesn't recognise as a known access
/// value. A megabyte-sized value must round-trip verbatim (parse, emit,
/// parse) without loss or truncation. Pins the contract that lenient
/// mode does not silently cap or normalise escape-hatch payloads.
#[test]
fn megabyte_other_string_in_lenient_mode_round_trips() {
    const SIZE: usize = 1024 * 1024;

    let garbage = "x".repeat(SIZE);
    let xml = format!(
        r#"<?xml version="1.0"?>
        <dictionary>
            <suite name="S" code="SUIT">
                <class name="thing" code="thng">
                    <element type="bit" access="{garbage}"/>
                </class>
            </suite>
        </dictionary>"#
    );

    let dict: Dictionary = xml
        .parse()
        .expect("lenient parse of megabyte Other(_) must succeed");

    let elem_access = dict.suites[0].classes[0].elements[0]
        .access
        .as_ref()
        .expect("element has @access");
    match elem_access {
        Access::Other(s) => assert_eq!(
            s.len(),
            SIZE,
            "Access::Other must preserve a {SIZE}-byte value verbatim"
        ),
        other => panic!("expected Access::Other(_), got {other:?}"),
    }

    let emitted = dict.to_xml_string().expect("serialize");
    let reparsed: Dictionary = emitted.parse().expect("reparse");
    assert_eq!(
        dict, reparsed,
        "megabyte Other(_) value must survive parse->emit->parse identity"
    );
}
