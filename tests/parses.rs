//! Integration tests for the public parsing API.
//!
//! These are the bare minimum to confirm `from_str`/`from_path` round-trip a
//! synthetic sdef into the expected AST. The specialist should grow this file
//! into a real test suite — at minimum, fixtures vendored from
//! `/System/Library/ScriptingDefinitions/` (which are Apple-published, not
//! application-proprietary) covering classes, enumerations, and the elements
//! the initial AST doesn't yet model.

mod common;

use std::path::PathBuf;

use sdef::{AccessorStyle, Dictionary};
// Bring FromStr into scope so tests can use `Dictionary::from_str` directly
// alongside the more-idiomatic `.parse::<Dictionary>()`.
#[allow(unused_imports)]
use std::str::FromStr;

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

#[test]
fn parses_classes_with_inheritance() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.classes.len(), 2);

    let shape = &suite.classes[0];
    assert_eq!(shape.name, "shape");
    assert_eq!(shape.code, "SYsh");
    assert_eq!(shape.plural.as_deref(), Some("shapes"));
    assert!(shape.inherits.is_none());
    let cocoa = shape.cocoa.as_ref().expect("shape has <cocoa>");
    assert_eq!(cocoa.class.as_deref(), Some("SyntheticShape"));
    assert_eq!(shape.properties.len(), 1);
    assert_eq!(shape.properties[0].name, "name");
    assert_eq!(shape.properties[0].access.as_deref(), Some("r"));

    let rect = &suite.classes[1];
    assert_eq!(rect.name, "rectangle");
    assert_eq!(rect.inherits.as_deref(), Some("shape"));
    assert_eq!(rect.types.len(), 1);
    assert_eq!(rect.types[0].ty, "shape");
    assert_eq!(rect.properties.len(), 2);
    // Property with nested <cocoa key="...">
    let height = &rect.properties[1];
    assert_eq!(height.name, "height");
    let height_cocoa = height.cocoa.as_ref().expect("height has <cocoa>");
    assert_eq!(height_cocoa.key.as_deref(), Some("heightInPoints"));
}

#[test]
fn parses_element_with_accessors() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let rect = &dict.suites[0].classes[1];
    assert_eq!(rect.elements.len(), 1);

    let elem = &rect.elements[0];
    assert_eq!(elem.ty, "point");
    assert_eq!(elem.access.as_deref(), Some("r"));
    let cocoa = elem.cocoa.as_ref().expect("element has <cocoa>");
    assert_eq!(cocoa.key.as_deref(), Some("anchorPoints"));

    assert_eq!(elem.accessors.len(), 2);
    assert_eq!(elem.accessors[0].style, AccessorStyle::Index);
    assert_eq!(elem.accessors[1].style, AccessorStyle::Id);
}

#[test]
fn parses_contents_and_responds_to() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let rect = &dict.suites[0].classes[1];

    assert_eq!(rect.contents.len(), 1);
    let contents = &rect.contents[0];
    assert_eq!(contents.name.as_deref(), Some("content"));
    assert_eq!(contents.code.as_deref(), Some("pcnt"));
    assert_eq!(contents.ty.as_deref(), Some("text"));

    assert_eq!(rect.responds_to.len(), 1);
    let rto = &rect.responds_to[0];
    assert_eq!(rto.command.as_deref(), Some("rotate"));
    assert!(rto.name.is_none());
    let cocoa = rto.cocoa.as_ref().expect("responds-to has <cocoa>");
    assert_eq!(cocoa.method.as_deref(), Some("rotateBy:"));
}

#[test]
fn parses_class_extension() {
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");

    let suite = &dict.suites[0];
    assert_eq!(suite.class_extensions.len(), 1);

    let ext = &suite.class_extensions[0];
    assert_eq!(ext.extends, "rectangle");
    assert_eq!(ext.title.as_deref(), Some("Annotations"));
    assert_eq!(ext.id.as_deref(), Some("rect-annotations"));
    assert_eq!(ext.properties.len(), 1);
    assert_eq!(ext.properties[0].name, "label");
}

// ===== Strict-mode regression tests =====
//
// These tests are the *load-bearing* drift detectors for the crate:
// - The synthetic-fixture-under-strict-mode test fails if anyone adds an
//   element to the fixture without also modelling it (or vice versa).
// - The MoneyMoney.sdef test (when present) cross-checks our DTD coverage
//   against a real-world consumer-shipped sdef.
// - The unknown-element / lenient-tolerates pair proves strict mode is
//   distinct from lenient mode and actually rejects what it claims to.

const VENDOR_EXTENDED_DICT: &str = r#"<?xml version="1.0"?>
<dictionary>
    <suite name="X" code="SUIT">
        <command name="foo" code="SUITfoo1">
            <vendor-extension/>
        </command>
    </suite>
</dictionary>"#;

#[test]
fn strict_mode_accepts_full_synthetic_fixture() {
    // Regression test: if anyone adds an unmodelled element to the
    // synthetic fixture (or removes a name from KNOWN_ELEMENTS without a
    // corresponding fixture update), this fails loud. Combined with the
    // other commit-N tests that read deep fields, it proves every element
    // we modelled is exercised at least once.
    let dict = Dictionary::from_path_strict(fixture("synthetic.sdef"))
        .expect("synthetic.sdef must parse cleanly under strict mode");
    // Spot-check: at least one element from each major DTD category is
    // present, so a future regression that drops a whole category from the
    // fixture would also fail here rather than silently shrinking coverage.
    let suite = &dict.suites[0];
    assert!(
        !suite.commands.is_empty(),
        "synthetic fixture lost its commands"
    );
    assert!(
        !suite.events.is_empty(),
        "synthetic fixture lost its events"
    );
    assert!(
        !suite.classes.is_empty(),
        "synthetic fixture lost its classes"
    );
    assert!(
        !suite.class_extensions.is_empty(),
        "synthetic fixture lost its class-extensions"
    );
    assert!(
        !suite.enumerations.is_empty(),
        "synthetic fixture lost its enumerations"
    );
    assert!(
        !suite.record_types.is_empty(),
        "synthetic fixture lost its record-types"
    );
    assert!(
        !suite.value_types.is_empty(),
        "synthetic fixture lost its value-types"
    );
    assert!(
        !suite.documentation.is_empty(),
        "synthetic fixture lost its documentation"
    );
}

#[test]
fn strict_mode_rejects_unknown_element_with_typed_payload() {
    let err = Dictionary::from_str_strict(VENDOR_EXTENDED_DICT)
        .expect_err("strict mode must reject <vendor-extension>");
    match &err {
        sdef::Error::UnknownElement { name } => {
            assert_eq!(
                name, "vendor-extension",
                "the unknown-element payload must carry the local name so callers can route on it"
            );
        }
        other => panic!("expected Error::UnknownElement, got {other:?}"),
    }
    // The Display impl must surface the offending element so humans reading
    // a CI log can diagnose the failure without re-running with --nocapture.
    let rendered = err.to_string();
    assert!(
        rendered.contains("vendor-extension"),
        "Error::Display must include the offending element name; got: {rendered}"
    );
}

#[test]
fn lenient_mode_silently_drops_unknown_element() {
    // Sibling to the strict test above — proves the two modes have
    // genuinely different behaviour on the same input. If quick-xml's
    // serde defaults ever changed to reject unknown children, this would
    // start failing and surface the surprise.
    let dict = VENDOR_EXTENDED_DICT
        .parse::<Dictionary>()
        .expect("lenient mode tolerates unknown child elements");
    let cmd = dict
        .command("foo")
        .expect("foo parses despite vendor-extension");
    assert_eq!(cmd.code, "SUITfoo1");
}

#[test]
fn strict_mode_rejects_xi_include_with_prefix() {
    let xml = r#"<?xml version="1.0"?>
<dictionary xmlns:xi="http://www.w3.org/2001/XInclude">
    <suite name="X" code="SUIT">
        <xi:include href="other.sdef"/>
    </suite>
</dictionary>"#;
    let err = Dictionary::from_str_strict(xml).expect_err("strict mode must reject <xi:include>");
    assert!(
        matches!(err, sdef::Error::XIncludeUnsupported),
        "expected Error::XIncludeUnsupported, got {err:?}"
    );
}

#[test]
fn strict_mode_forwards_malformed_xml_errors_as_xml_variant() {
    // Strict mode must not silently swallow parse errors. A truncated
    // document surfaces as Error::Xml, not Error::UnknownElement.
    let err =
        Dictionary::from_str_strict("<dictionary").expect_err("truncated document must error");
    assert!(
        matches!(err, sdef::Error::Xml(_)),
        "malformed XML must surface as Error::Xml under strict mode, got {err:?}"
    );
}

#[test]
fn strict_mode_accepts_money_money_sdef_when_present() {
    // Cross-checks our DTD coverage against a real-world sdef. Skips
    // cleanly on machines where MoneyMoney.app isn't installed (CI).
    let path = "/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef";
    if !std::path::Path::new(path).exists() {
        eprintln!("(MoneyMoney.app not installed; skipping real-world strict-mode check)");
        return;
    }
    let dict = Dictionary::from_path_strict(path).expect(
        "MoneyMoney.sdef must parse cleanly under strict mode \
         — DTD coverage gap in this crate if it doesn't",
    );
    // Spot-check: real sdef has the data we expect. Catches the case where
    // a strict-mode change accidentally returns an empty Dictionary.
    assert!(!dict.suites.is_empty());
    assert!(!dict.suites[0].commands.is_empty());
}

#[test]
fn command_result_rename_is_publicly_exposed() {
    // Sanity smoke that `Result_` → `CommandResult` rename is reachable
    // via the public API and the type matches Command.result's element.
    let dict = Dictionary::from_path(fixture("synthetic.sdef")).expect("parses");
    let echo = dict.command("echo text").expect("command must exist");
    let result: &sdef::CommandResult = echo.result.as_ref().expect("echo has a result");
    assert_eq!(result.ty.as_deref(), Some("text"));
}

// ===== Fixture-coverage regression tests =====

#[test]
fn parses_mini_fixture_with_minimum_valid_content() {
    // The minimum-valid sdef — one suite, one command, no optional attrs.
    // Catches regressions where a refactor accidentally requires more.
    let dict = Dictionary::from_path_strict(fixture("mini.sdef")).expect("parses");

    assert!(dict.title.is_none());
    assert_eq!(dict.suites.len(), 1);
    let suite = &dict.suites[0];
    assert_eq!(suite.name, "Mini");
    assert_eq!(suite.code, "MINI");
    assert!(suite.description.is_none());
    assert!(!suite.hidden);
    assert!(suite.cocoa.is_none());
    assert!(suite.access_groups.is_empty());
    assert!(suite.documentation.is_empty());
    assert!(suite.events.is_empty());
    assert!(suite.classes.is_empty());

    assert_eq!(suite.commands.len(), 1);
    let cmd = &suite.commands[0];
    assert_eq!(cmd.name, "ping");
    assert_eq!(cmd.code, "MINIping");
    assert!(cmd.id.is_none());
    assert!(cmd.description.is_none());
    assert!(!cmd.hidden);
    assert!(cmd.cocoa.is_none());
    assert!(cmd.access_groups.is_empty());
    assert!(cmd.synonyms.is_empty());
    assert!(cmd.documentation.is_empty());
    assert!(cmd.parameters.is_empty());
    assert!(cmd.direct_parameter.is_none());
    assert!(cmd.result.is_none());
    assert!(cmd.xrefs.is_empty());
}

#[test]
fn parses_extras_multiple_synonyms_and_inline_parameter_doc() {
    let dict = Dictionary::from_path_strict(fixture("extras.sdef")).expect("parses");
    let multi = dict.command("multi").expect("multi command exists");

    // Multiple <synonym> siblings on one command.
    assert_eq!(multi.synonyms.len(), 2);
    assert_eq!(multi.synonyms[0].name.as_deref(), Some("multiple"));
    assert!(multi.synonyms[0].code.is_none());
    assert_eq!(multi.synonyms[1].name.as_deref(), Some("many"));
    assert_eq!(multi.synonyms[1].code.as_deref(), Some("MULT"));

    // <documentation> inside <parameter> — a 10.10 placement that wasn't
    // exercised in synthetic.sdef.
    let value = multi
        .parameters
        .iter()
        .find(|p| p.name == "value")
        .expect("`value` parameter exists");
    assert_eq!(value.documentation.len(), 1);
    assert_eq!(value.documentation[0].html.len(), 1);
    assert_eq!(
        value.documentation[0].html[0],
        "Description embedded inside the parameter (since OS X 10.10)."
    );
    // Type child still parses alongside the documentation child.
    assert_eq!(value.types.len(), 1);
    assert_eq!(value.types[0].ty, "text");
}

#[test]
fn parses_extras_union_result_type() {
    // Result with three sibling <type> children (union expression).
    let dict = Dictionary::from_path_strict(fixture("extras.sdef")).expect("parses");
    let multi = dict.command("multi").expect("multi command exists");
    let result = multi.result.as_ref().expect("multi has a result");

    assert!(
        result.ty.is_none(),
        "union form is mutually exclusive with type attribute"
    );
    assert_eq!(result.types.len(), 3);
    assert_eq!(result.types[0].ty, "integer");
    assert_eq!(result.types[1].ty, "real");
    assert_eq!(result.types[2].ty, "boolean");
    for t in &result.types {
        assert!(!t.list, "no union member is marked list in this fixture");
    }
}

#[test]
fn parses_extras_enumerator_with_cocoa_string_value() {
    let dict = Dictionary::from_path_strict(fixture("extras.sdef")).expect("parses");
    let severity = dict
        .suites
        .iter()
        .flat_map(|s| &s.enumerations)
        .find(|e| e.name == "severity")
        .expect("severity enumeration exists");

    assert_eq!(severity.enumerators.len(), 3);
    let info = &severity.enumerators[0];
    assert_eq!(info.name, "info");
    let cocoa = info.cocoa.as_ref().expect("enumerator has <cocoa>");
    assert_eq!(cocoa.string_value.as_deref(), Some("info-level"));
    assert!(cocoa.class.is_none());
    assert!(cocoa.integer_value.is_none());
    assert!(cocoa.boolean_value.is_none());
}

#[test]
fn parses_extras_class_with_interleaved_children() {
    // The fixture's <class name="entry"> is intentionally
    // property → element → property → responds-to, not contiguous-by-type.
    // Guards against regressions in interleaved-children handling.
    let dict = Dictionary::from_path_strict(fixture("extras.sdef")).expect("parses");
    let entry = dict
        .suites
        .iter()
        .flat_map(|s| &s.classes)
        .find(|c| c.name == "entry")
        .expect("entry class exists");

    assert_eq!(
        entry.properties.len(),
        2,
        "both properties survive interleaving"
    );
    let prop_names: Vec<&str> = entry.properties.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(prop_names, ["severity", "message"]);

    assert_eq!(entry.elements.len(), 1, "the element survives interleaving");
    assert_eq!(entry.elements[0].ty, "entry");
    assert_eq!(
        entry.responds_to.len(),
        1,
        "the responds-to survives interleaving"
    );
}

#[test]
fn suite_with_interleaved_commands_and_enumerations_parses() {
    // Mirror-image of the class-level interleaving test, at suite scope.
    // Apple's own CocoaStandard.sdef alternates commands and enumerations,
    // which is what surfaced the underlying quick-xml limitation.
    let xml = r#"<?xml version="1.0"?>
<dictionary>
    <suite name="X" code="SUIT">
        <command name="c1" code="SUITcmd1"/>
        <enumeration name="e1" code="enm1">
            <enumerator name="a" code="aaaa"/>
        </enumeration>
        <command name="c2" code="SUITcmd2"/>
        <enumeration name="e2" code="enm2">
            <enumerator name="b" code="bbbb"/>
        </enumeration>
        <command name="c3" code="SUITcmd3"/>
    </suite>
</dictionary>"#;
    let dict: Dictionary = xml.parse().expect("interleaved suite parses");
    let suite = &dict.suites[0];

    assert_eq!(
        suite.commands.len(),
        3,
        "all three commands survive interleaving"
    );
    let cmd_names: Vec<&str> = suite.commands.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(cmd_names, ["c1", "c2", "c3"]);

    assert_eq!(
        suite.enumerations.len(),
        2,
        "both enumerations survive interleaving"
    );
    let enum_names: Vec<&str> = suite.enumerations.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(enum_names, ["e1", "e2"]);
}

#[test]
fn parses_extras_property_access_group_and_legacy_responds_to_name() {
    let dict = Dictionary::from_path_strict(fixture("extras.sdef")).expect("parses");
    let entry = dict
        .suites
        .iter()
        .flat_map(|s| &s.classes)
        .find(|c| c.name == "entry")
        .expect("entry class exists");

    // <access-group> nested inside a <property> (placement not exercised
    // in synthetic.sdef — synthetic only puts access-groups at the
    // suite/command level).
    let severity = entry
        .properties
        .iter()
        .find(|p| p.name == "severity")
        .expect("severity property exists");
    assert_eq!(severity.access_groups.len(), 1);
    assert_eq!(
        severity.access_groups[0].identifier,
        "com.example.edge.read"
    );
    assert_eq!(severity.access_groups[0].access.as_deref(), Some("r"));

    // <element> with four accessor styles (exercises the accessor vector,
    // not just one entry as synthetic.sdef does).
    assert_eq!(entry.elements.len(), 1);
    let elem = &entry.elements[0];
    assert_eq!(elem.accessors.len(), 4);
    let styles: Vec<&str> = elem.accessors.iter().map(|a| a.style.as_str()).collect();
    assert_eq!(styles, ["index", "name", "id", "range"]);

    // <responds-to> with both new `command` and legacy `name` attributes
    // present — confirms the backward-compat field deserializes.
    assert_eq!(entry.responds_to.len(), 1);
    let rto = &entry.responds_to[0];
    assert_eq!(rto.command.as_deref(), Some("rotate"));
    assert_eq!(rto.name.as_deref(), Some("rotate"));
    let cocoa = rto.cocoa.as_ref().expect("responds-to carries <cocoa>");
    assert_eq!(cocoa.method.as_deref(), Some("rotateLog:"));
}

// ===== Opt-in real-world corpus probe =====
//
// Run with: SDEF_FIXTURE_DIR=/System/Library/ScriptingDefinitions:/Applications/Xcode.app/Contents/Resources \
//           cargo test corpus_smoke -- --ignored --nocapture
//
// Iterates every *.sdef under the given (colon-separated) directories and
// asserts they all parse cleanly under strict mode. Emits a coverage report
// to `target/corpus_coverage.json` so the conformance-matrix generator can
// surface which DTD constructs are exercised by real-world fixtures.

#[test]
#[ignore = "opt-in: requires SDEF_FIXTURE_DIR env var"]
fn corpus_smoke() {
    let Ok(raw) = std::env::var("SDEF_FIXTURE_DIR") else {
        eprintln!(
            "SDEF_FIXTURE_DIR not set — this test is opt-in. Try: \
             SDEF_FIXTURE_DIR=/System/Library/ScriptingDefinitions \
             cargo test corpus_smoke -- --ignored --nocapture"
        );
        return;
    };

    let dirs: Vec<PathBuf> = raw
        .split(':')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();

    let mut count = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut fixtures: std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, std::collections::BTreeSet<String>>,
    > = std::collections::BTreeMap::new();

    for dir in &dirs {
        let read = match std::fs::read_dir(dir) {
            Ok(r) => r,
            Err(e) => {
                // Missing directories are not an error — e.g. Xcode may not
                // be installed. Log and continue so the test reflects what
                // the runner actually has available.
                eprintln!("corpus_smoke: skipping {}: {e}", dir.display());
                continue;
            }
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "sdef") {
                continue;
            }
            count += 1;
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    failures.push(format!("{}: read failed: {e}", path.display()));
                    continue;
                }
            };
            match sdef::Dictionary::from_str_strict(&text) {
                Ok(_) => {
                    fixtures.insert(path.display().to_string(), common::scan_xml_coverage(&text));
                }
                Err(e) => failures.push(format!("{}: {e}", path.display())),
            }
        }
    }

    let coverage = common::CorpusCoverage::from_fixtures(fixtures);
    if let Err(e) = coverage.write_default() {
        eprintln!(
            "corpus_smoke: failed to write {}: {e}",
            common::CORPUS_COVERAGE_PATH
        );
    } else {
        eprintln!(
            "corpus_smoke: wrote coverage to {} ({} elements observed across {} fixtures)",
            common::CORPUS_COVERAGE_PATH,
            coverage.aggregate.len(),
            coverage.total_fixtures,
        );
    }

    eprintln!(
        "corpus_smoke: checked {count} sdef file(s) under {} directory(ies)",
        dirs.len()
    );
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("  FAIL: {f}");
        }
        panic!(
            "{} of {} sdef file(s) in corpus failed strict parsing",
            failures.len(),
            count
        );
    }
}
