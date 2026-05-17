//! Strict-mode pre-validation pass for [`crate::Dictionary::from_str_strict`]
//! and friends.
//!
//! quick-xml's serde layer silently drops unknown XML child elements, so a
//! permissive parse cannot tell a real-world sdef apart from one that uses
//! elements this crate does not model. Strict mode runs a fast event-level
//! pre-pass that rejects unknown element names (signalling DTD drift or
//! vendor-specific extensions) and `<xi:include>` directives (which we don't
//! resolve and would otherwise be silently treated as no-ops).
//!
//! The list of accepted element names is the complete set defined in
//! `/System/Library/DTDs/sdef.dtd` as of the macOS release this crate was
//! modelled against. The hash-pinned drift test in `tests/dtd_drift.rs`
//! checks the live system DTD against that snapshot and fails when Apple
//! makes additions; that's the signal to update the list below (and the
//! rest of the AST) in lock-step.
//!
//! Strict mode also rejects unknown values for the eight closed-enum
//! attributes (`accessor.style`, `<…>.access`, `<…>.requires-access`,
//! `cocoa.boolean-value`) declared by the DTD. The pinned value sets live
//! in [`CLOSED_ENUM_ATTRS`] below and are kept in lock-step with
//! `tests/fixtures/attribute_manifest.toml` via the `attribute_conformance`
//! test.
//!
//! Limitations: this pass validates element names plus the eight
//! closed-enum attributes, but not arbitrary attribute names. For full DTD
//! validation rely on `xmllint --dtdvalid` (wired up in
//! `tests/dtd_drift.rs` on macOS).

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::Error;

/// Every element name accepted by strict mode. Sorted for `binary_search`.
///
/// Must contain exactly the element types modelled by the crate. Adding or
/// removing an entry without a corresponding AST change is a load-bearing
/// mistake — the `known_elements_count_matches_modeled_dtd` unit test
/// below pins the length so the change is forced through review.
pub(crate) const KNOWN_ELEMENTS: &[&str] = &[
    "access-group",
    "accessor",
    "class",
    "class-extension",
    "cocoa",
    "command",
    "contents",
    "dictionary",
    "direct-parameter",
    "documentation",
    "element",
    "enumeration",
    "enumerator",
    "event",
    "html",
    "parameter",
    "property",
    "record-type",
    "responds-to",
    "result",
    "suite",
    "synonym",
    "type",
    "value-type",
    "xref",
];

/// Closed-enumeration attribute constraint. Each entry pins one
/// `(element, attribute) → allowed values` mapping from the DTD; strict
/// mode rejects any value outside the set.
///
/// The values are kept in lock-step with the DTD via the
/// `attribute_conformance` integration test, which parses the live
/// `/System/Library/DTDs/sdef.dtd` via libxml2 and diff-checks against
/// `tests/fixtures/attribute_manifest.toml`. Updating either table
/// without the other will surface a CI failure on macOS.
struct ClosedEnumAttr {
    element: &'static str,
    attribute: &'static [u8],
    allowed: &'static [&'static [u8]],
}

const CLOSED_ENUM_ATTRS: &[ClosedEnumAttr] = &[
    ClosedEnumAttr {
        element: "accessor",
        attribute: b"style",
        allowed: &[b"index", b"name", b"id", b"range", b"relative", b"test"],
    },
    ClosedEnumAttr {
        element: "access-group",
        attribute: b"access",
        allowed: &[b"r", b"w", b"rw"],
    },
    ClosedEnumAttr {
        element: "cocoa",
        attribute: b"boolean-value",
        allowed: &[b"YES", b"NO"],
    },
    ClosedEnumAttr {
        element: "contents",
        attribute: b"access",
        allowed: &[b"r", b"w", b"rw"],
    },
    ClosedEnumAttr {
        element: "direct-parameter",
        attribute: b"requires-access",
        allowed: &[b"r", b"w", b"rw"],
    },
    ClosedEnumAttr {
        element: "element",
        attribute: b"access",
        allowed: &[b"r", b"w", b"rw"],
    },
    ClosedEnumAttr {
        element: "parameter",
        attribute: b"requires-access",
        allowed: &[b"r", b"w", b"rw"],
    },
    ClosedEnumAttr {
        element: "property",
        attribute: b"access",
        allowed: &[b"r", b"w", b"rw"],
    },
];

/// Walk `xml` once, rejecting unknown element names, `xi:include`
/// directives, and out-of-range values for the closed-enum attributes
/// listed in [`CLOSED_ENUM_ATTRS`]. Returns `Ok(())` for documents that
/// contain only modelled constructs; callers then proceed with regular
/// serde-driven deserialization.
pub(crate) fn validate_strict(xml: &str) -> Result<(), Error> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let qname = e.name();
                let prefix = qname.prefix();
                let local = qname.local_name();
                let local_str =
                    std::str::from_utf8(local.as_ref()).map_err(|_| Error::UnknownElement {
                        name: format!("{:?}", local.as_ref()),
                    })?;
                if prefix.is_some_and(|p| p.as_ref() == b"xi") {
                    return Err(Error::XIncludeUnsupported);
                }
                if KNOWN_ELEMENTS.binary_search(&local_str).is_err() {
                    return Err(Error::UnknownElement {
                        name: local_str.to_owned(),
                    });
                }
                validate_closed_enum_attrs(local_str, &e)?;
            }
            Ok(Event::Eof) => return Ok(()),
            Err(e) => {
                // Wrap the quick-xml error so callers see a typed
                // ::Xml variant rather than a panic.
                return Err(Error::Xml(quick_xml::DeError::Custom(e.to_string())));
            }
            _ => {}
        }
    }
}

/// Inspect the attributes of one start/empty element against
/// [`CLOSED_ENUM_ATTRS`]. Cheap: at most 8 constraint rows checked per
/// element, and attribute iteration short-circuits on the first match.
fn validate_closed_enum_attrs(
    local_str: &str,
    e: &quick_xml::events::BytesStart<'_>,
) -> Result<(), Error> {
    for constraint in CLOSED_ENUM_ATTRS {
        if constraint.element != local_str {
            continue;
        }
        for attr in e.attributes().flatten() {
            let key = attr.key;
            if key.prefix().is_some() {
                continue; // ignore namespaced attributes (xmlns:xi etc.)
            }
            if key.local_name().as_ref() != constraint.attribute {
                continue;
            }
            let value = attr.value.as_ref();
            if !constraint.allowed.contains(&value) {
                return Err(Error::UnknownAttributeValue {
                    element: constraint.element.to_owned(),
                    attribute: String::from_utf8_lossy(constraint.attribute).into_owned(),
                    value: String::from_utf8_lossy(value).into_owned(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_elements_count_matches_modeled_dtd() {
        // /System/Library/DTDs/sdef.dtd defines 25 element types at the
        // modelled macOS release. Adding to KNOWN_ELEMENTS without also
        // teaching the AST about the element is the load-bearing mistake
        // this assertion exists to prevent — bumping this number must be
        // a deliberate, reviewed action accompanied by an AST change.
        assert_eq!(KNOWN_ELEMENTS.len(), 25);
    }

    #[test]
    fn known_elements_are_sorted_for_binary_search() {
        let mut sorted: Vec<&str> = KNOWN_ELEMENTS.to_vec();
        sorted.sort_unstable();
        assert_eq!(
            KNOWN_ELEMENTS,
            sorted.as_slice(),
            "KNOWN_ELEMENTS must stay sorted: binary_search relies on it"
        );
    }

    #[test]
    fn known_elements_have_no_duplicates() {
        let mut seen: Vec<&str> = KNOWN_ELEMENTS.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            KNOWN_ELEMENTS.len(),
            "duplicates in KNOWN_ELEMENTS"
        );
    }

    #[test]
    fn validate_accepts_minimal_valid_dictionary() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary>
                <suite name="S" code="SUIT">
                    <command name="c" code="SUITcmd1"/>
                </suite>
            </dictionary>"#;
        validate_strict(xml).expect("minimal valid dictionary must pass strict mode");
    }

    #[test]
    fn validate_rejects_unknown_element_with_local_name() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary>
                <suite name="S" code="SUIT">
                    <vendor-thing/>
                </suite>
            </dictionary>"#;
        match validate_strict(xml) {
            Err(Error::UnknownElement { name }) => {
                assert_eq!(name, "vendor-thing");
            }
            other => panic!("expected UnknownElement, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_xi_include_by_prefix() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary xmlns:xi="http://www.w3.org/2001/XInclude">
                <suite name="S" code="SUIT">
                    <xi:include href="other.sdef"/>
                </suite>
            </dictionary>"#;
        match validate_strict(xml) {
            Err(Error::XIncludeUnsupported) => {}
            other => panic!("expected XIncludeUnsupported, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_unknown_accessor_style() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary>
                <suite name="S" code="SUIT">
                    <class name="thing" code="thng">
                        <element type="bit">
                            <accessor style="weird"/>
                        </element>
                    </class>
                </suite>
            </dictionary>"#;
        match validate_strict(xml) {
            Err(Error::UnknownAttributeValue {
                element,
                attribute,
                value,
            }) => {
                assert_eq!(element, "accessor");
                assert_eq!(attribute, "style");
                assert_eq!(value, "weird");
            }
            other => panic!("expected UnknownAttributeValue, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_unknown_access_value() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary>
                <suite name="S" code="SUIT">
                    <class name="thing" code="thng">
                        <element type="bit" access="execute"/>
                    </class>
                </suite>
            </dictionary>"#;
        match validate_strict(xml) {
            Err(Error::UnknownAttributeValue {
                element,
                attribute,
                value,
            }) => {
                assert_eq!(element, "element");
                assert_eq!(attribute, "access");
                assert_eq!(value, "execute");
            }
            other => panic!("expected UnknownAttributeValue, got {other:?}"),
        }
    }

    #[test]
    fn validate_accepts_canonical_closed_enum_values() {
        let xml = r#"<?xml version="1.0"?>
            <dictionary>
                <suite name="S" code="SUIT">
                    <class name="thing" code="thng">
                        <element type="bit" access="rw">
                            <accessor style="index"/>
                        </element>
                    </class>
                </suite>
            </dictionary>"#;
        validate_strict(xml).expect("canonical closed-enum values must pass strict mode");
    }
}
