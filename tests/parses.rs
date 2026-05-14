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

#[test]
fn parses_cocoa_on_suite_and_command() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    let suite_cocoa = suite.cocoa.as_ref().expect("suite has <cocoa>");
    assert_eq!(suite_cocoa.name.as_deref(), Some("SyntheticSuite"));
    assert!(suite_cocoa.class.is_none());

    let echo = dict.command("echo text").expect("command must exist");
    let cmd_cocoa = echo.cocoa.as_ref().expect("command has <cocoa>");
    assert_eq!(cmd_cocoa.class.as_deref(), Some("EchoHandler"));
    assert!(cmd_cocoa.name.is_none());

    // Commands without <cocoa> deserialise to None.
    let add = dict.command("add numbers").expect("command must exist");
    assert!(add.cocoa.is_none());
}

#[test]
fn parses_access_groups() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.access_groups.len(), 1);
    assert_eq!(
        suite.access_groups[0].identifier,
        "com.example.synthetic.read"
    );
    assert_eq!(suite.access_groups[0].access.as_deref(), Some("r"));

    let echo = dict.command("echo text").expect("command must exist");
    assert_eq!(echo.access_groups.len(), 1);
    assert_eq!(
        echo.access_groups[0].identifier,
        "com.example.synthetic.read"
    );
    assert!(echo.access_groups[0].access.is_none());

    // Commands without <access-group> deserialise to an empty vec.
    let add = dict.command("add numbers").expect("command must exist");
    assert!(add.access_groups.is_empty());
}

#[test]
fn parses_command_id_and_xref() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let echo = dict.command("echo text").expect("command must exist");
    assert_eq!(echo.id.as_deref(), Some("echo-cmd"));
    assert_eq!(echo.xrefs.len(), 1);
    assert_eq!(echo.xrefs[0].target, "add-numbers");
    assert!(!echo.xrefs[0].hidden);

    let add = dict.command("add numbers").expect("command must exist");
    assert_eq!(add.id.as_deref(), Some("add-numbers"));
    assert!(add.xrefs.is_empty());
}

#[test]
fn parses_synonym_on_command() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let echo = dict.command("echo text").expect("command must exist");
    assert_eq!(echo.synonyms.len(), 1);
    assert_eq!(echo.synonyms[0].name.as_deref(), Some("repeat text"));
    assert!(echo.synonyms[0].code.is_none());
    assert!(!echo.synonyms[0].hidden);
}

#[test]
fn parses_documentation_blocks() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.documentation.len(), 1);
    assert_eq!(suite.documentation[0].html.len(), 1);
    assert_eq!(
        suite.documentation[0].html[0],
        "Top-level documentation for the synthetic suite."
    );

    let echo = dict.command("echo text").expect("command must exist");
    assert_eq!(echo.documentation.len(), 1);
    assert_eq!(echo.documentation[0].html.len(), 2);
    assert_eq!(
        echo.documentation[0].html[0],
        "Echoes the supplied text back to the caller."
    );
    assert_eq!(
        echo.documentation[0].html[1],
        "Optionally upper-cases the result."
    );
}

#[test]
fn parses_type_child_elements_with_list_and_union() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let add = dict.command("add numbers").expect("command must exist");

    // The `and` parameter declares a union: <type>real</type> | <type list="yes">real</type>.
    let and_param = add
        .parameters
        .iter()
        .find(|p| p.name == "and")
        .expect("`and` parameter must exist");
    assert!(
        and_param.ty.is_none(),
        "type attribute and <type> children are mutually exclusive in this fixture"
    );
    assert_eq!(and_param.types.len(), 2);
    assert_eq!(and_param.types[0].ty, "real");
    assert!(!and_param.types[0].list);
    assert_eq!(and_param.types[1].ty, "real");
    assert!(and_param.types[1].list);

    // The result uses a single <type> child instead of the attribute.
    let result = add.result.as_ref().expect("result must exist");
    assert!(result.ty.is_none());
    assert_eq!(result.types.len(), 1);
    assert_eq!(result.types[0].ty, "real");
    assert!(!result.types[0].list);
}

#[test]
fn parses_hidden_and_requires_access_flags() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert!(!suite.hidden);

    let echo = dict.command("echo text").expect("command must exist");
    assert!(!echo.hidden);

    let text_param = &echo.parameters[0];
    assert_eq!(text_param.requires_access.as_deref(), Some("r"));
    assert!(!text_param.hidden);

    let upper_param = &echo.parameters[1];
    assert!(!upper_param.hidden); // explicit hidden="no" in fixture
    assert!(upper_param.requires_access.is_none());
}

#[test]
fn parses_event() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.events.len(), 1);

    let opened = &suite.events[0];
    assert_eq!(opened.name, "opened");
    assert_eq!(opened.code, "SYNTeopn");
    assert_eq!(opened.id.as_deref(), Some("opened-event"));
    assert_eq!(
        opened.description.as_deref(),
        Some("Fired when a synthetic document is opened.")
    );
    let cocoa = opened.cocoa.as_ref().expect("event has <cocoa>");
    assert_eq!(cocoa.class.as_deref(), Some("OpenedEventHandler"));
    assert_eq!(opened.parameters.len(), 1);
    assert_eq!(opened.parameters[0].name, "reference");
}

#[test]
fn parses_enumeration_with_enumerators() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.enumerations.len(), 1);

    let format = &suite.enumerations[0];
    assert_eq!(format.name, "export format");
    assert_eq!(format.code, "SYfm");
    assert_eq!(format.inline.as_deref(), Some("2"));
    assert_eq!(format.enumerators.len(), 3);

    assert_eq!(format.enumerators[0].name, "csv");
    assert_eq!(format.enumerators[0].code, "cmma");
    assert!(!format.enumerators[0].hidden);

    assert_eq!(format.enumerators[1].name, "json");
    assert_eq!(format.enumerators[2].name, "xml");
    assert!(format.enumerators[2].hidden); // hidden="yes" in fixture
}

#[test]
fn parses_record_type_with_properties() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.record_types.len(), 1);

    let bbox = &suite.record_types[0];
    assert_eq!(bbox.name, "bounding box");
    assert_eq!(bbox.code, "SYbx");
    assert_eq!(bbox.plural.as_deref(), Some("bounding boxes"));
    assert_eq!(bbox.properties.len(), 4);

    let left = &bbox.properties[0];
    assert_eq!(left.name, "left");
    assert_eq!(left.code, "left");
    assert_eq!(left.ty.as_deref(), Some("real"));
    assert!(left.access.is_none()); // attribute omitted → default "rw"
    assert!(left.in_properties.is_none()); // omitted → DTD-default "yes"

    let bottom = &bbox.properties[3];
    assert_eq!(bottom.name, "bottom");
    assert_eq!(bottom.access.as_deref(), Some("r"));
    assert_eq!(bottom.in_properties, Some(false)); // explicit "no" in fixture
}

#[test]
fn parses_value_type_with_synonym() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.value_types.len(), 1);

    let color = &suite.value_types[0];
    assert_eq!(color.name, "color");
    assert_eq!(color.code, "SYcl");
    let cocoa = color.cocoa.as_ref().expect("value-type has <cocoa>");
    assert_eq!(cocoa.class.as_deref(), Some("NSColor"));
    assert_eq!(color.synonyms.len(), 1);
    assert_eq!(color.synonyms[0].name.as_deref(), Some("colour"));
}
