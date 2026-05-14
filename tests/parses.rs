//! Integration tests for the public parsing API.
//!
//! These are the bare minimum to confirm `from_str`/`from_path` round-trip a
//! synthetic sdef into the expected AST. The specialist should grow this file
//! into a real test suite — at minimum, fixtures vendored from
//! `/System/Library/ScriptingDefinitions/` (which are Apple-published, not
//! application-proprietary) covering classes, enumerations, and the elements
//! the initial AST doesn't yet model.

use std::path::PathBuf;

use sdef::Dictionary;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn parses_synthetic_dictionary() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    assert_eq!(dict.title.as_deref(), Some("Synthetic Test Suite"));
    assert_eq!(dict.suites.len(), 1);

    let suite = &dict.suites[0];
    assert_eq!(suite.name, "Synthetic Suite");
    assert_eq!(suite.code, "SYNT");
    assert_eq!(suite.commands.len(), 2);
}

#[test]
fn looks_up_command_by_name() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let echo = dict.command("echo text").expect("command must exist");
    assert_eq!(echo.code, "SYNTecho");
    assert_eq!(echo.parameters.len(), 2);

    let text_param = &echo.parameters[0];
    assert_eq!(text_param.name, "text");
    assert_eq!(text_param.code, "text");
    assert_eq!(text_param.ty.as_deref(), Some("text"));
    assert!(!text_param.optional);

    let upper_param = &echo.parameters[1];
    assert_eq!(upper_param.name, "upper");
    assert!(upper_param.optional);

    assert!(echo.result.is_some());
    assert!(echo.direct_parameter.is_none());
}

#[test]
fn direct_parameter_is_parsed() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let add = dict.command("add numbers").expect("command must exist");
    let direct = add
        .direct_parameter
        .as_ref()
        .expect("add numbers takes a direct parameter");
    assert_eq!(direct.ty.as_deref(), Some("real"));
}

#[test]
fn unknown_command_returns_none() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");
    assert!(dict.command("not a command").is_none());
}

#[test]
fn malformed_xml_surfaces_error() {
    let err: sdef::Error = "<dictionary".parse::<Dictionary>().expect_err("must fail");
    // We just assert the variant — the underlying quick-xml message is not
    // a stable contract.
    assert!(matches!(err, sdef::Error::Xml(_)));
}
