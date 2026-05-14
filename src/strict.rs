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
//! modelled against. The hash-pinned drift test in commit 6's
//! `tests/dtd_drift.rs` checks the live system DTD against that snapshot and
//! fails when Apple makes additions; that's the signal to update the list
//! below (and the rest of the AST) in lock-step.
//!
//! Limitations: this pass validates element names only, not attribute names
//! or attribute value enumerations. For full DTD validation rely on
//! `xmllint --dtdvalid` (also wired up in commit 6's drift test on macOS).

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::Error;

/// Every element name accepted by strict mode. Sorted for `binary_search`.
///
/// Must contain exactly the element types modelled by the crate. Adding or
/// removing an entry without a corresponding AST change is a load-bearing
/// mistake — the `strict_mode_known_element_count_is_stable` integration
/// test in `tests/parses.rs` pins the length so the change is forced
/// through review.
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

/// Walk `xml` once, rejecting unknown element names and `xi:include`
/// directives. Returns `Ok(())` for documents that contain only modelled
/// elements; callers then proceed with regular serde-driven deserialization.
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
}
