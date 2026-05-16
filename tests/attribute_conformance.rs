//! Attribute-level conformance test.
//!
//! Cross-checks `tests/fixtures/attribute_manifest.toml` against the live
//! `/System/Library/DTDs/sdef.dtd`. On macOS this runs in full; on Linux/CI
//! hosts without the system DTD it self-skips.
//!
//! When this test fails, treat the diff in the panic message as a punch list:
//! update the manifest **and** add the missing field to the corresponding
//! struct under `src/`. The manifest's trust model mirrors
//! `src/strict.rs::KNOWN_ELEMENTS` — element/attribute changes must be
//! deliberate, paired, and reviewed.

mod common;

use std::collections::BTreeSet;

use common::{Manifest, parse_dtd, system_dtd_path};

#[test]
fn manifest_matches_live_dtd() {
    let Some(path) = system_dtd_path() else {
        eprintln!("skipping: system DTD absent (non-macOS host)");
        return;
    };
    let dtd = parse_dtd(path);
    let manifest = Manifest::load();

    let mut failures = Vec::<String>::new();

    // 1. Element-name sets must match exactly.
    let manifest_names: BTreeSet<&str> =
        manifest.elements.iter().map(|e| e.name.as_str()).collect();
    let dtd_names: BTreeSet<&str> = dtd.keys().map(String::as_str).collect();
    if manifest_names != dtd_names {
        let extra_manifest: Vec<&&str> = manifest_names.difference(&dtd_names).collect();
        let extra_dtd: Vec<&&str> = dtd_names.difference(&manifest_names).collect();
        failures.push(format!(
            "element-name set mismatch\n  extra in manifest: {extra_manifest:?}\n  extra in DTD:      {extra_dtd:?}"
        ));
    }

    // 2. Per-element attribute + required sets.
    for spec in &manifest.elements {
        let Some(dtd_attrs) = dtd.get(&spec.name) else {
            continue; // already reported above
        };
        let manifest_attrs: BTreeSet<&str> = spec.attributes.iter().map(String::as_str).collect();
        let dtd_attr_set: BTreeSet<&str> = dtd_attrs.iter().map(|a| a.name.as_str()).collect();
        if manifest_attrs != dtd_attr_set {
            let extra_m: Vec<&&str> = manifest_attrs.difference(&dtd_attr_set).collect();
            let extra_d: Vec<&&str> = dtd_attr_set.difference(&manifest_attrs).collect();
            failures.push(format!(
                "<{}>: attribute set mismatch\n  extra in manifest: {extra_m:?}\n  extra in DTD:      {extra_d:?}",
                spec.name
            ));
        }

        let manifest_req: BTreeSet<&str> = spec.required.iter().map(String::as_str).collect();
        let dtd_req: BTreeSet<&str> = dtd_attrs
            .iter()
            .filter(|a| a.required)
            .map(|a| a.name.as_str())
            .collect();
        if manifest_req != dtd_req {
            let extra_m: Vec<&&str> = manifest_req.difference(&dtd_req).collect();
            let extra_d: Vec<&&str> = dtd_req.difference(&manifest_req).collect();
            failures.push(format!(
                "<{}>: required-attribute set mismatch\n  extra in manifest: {extra_m:?}\n  extra in DTD:      {extra_d:?}",
                spec.name
            ));
        }
    }

    // 3. Enum values. The manifest enumerates every closed-enum (element,
    //    attribute) it cares about; yorn-typed (yes|no) attributes are
    //    handled by the yorn deserializer and intentionally excluded.
    let yorn: BTreeSet<&str> = ["yes", "no"].into_iter().collect();
    let mut dtd_enum_keys = BTreeSet::<String>::new();
    for (elem, attrs) in &dtd {
        for a in attrs {
            if a.enum_values.is_empty() {
                continue;
            }
            let values_set: BTreeSet<&str> = a.enum_values.iter().map(String::as_str).collect();
            if values_set == yorn {
                continue;
            }
            dtd_enum_keys.insert(format!("{elem}.{}", a.name));
        }
    }
    let manifest_enum_keys: BTreeSet<String> = manifest.enums.keys().cloned().collect();
    if manifest_enum_keys != dtd_enum_keys {
        let extra_m: Vec<&String> = manifest_enum_keys.difference(&dtd_enum_keys).collect();
        let extra_d: Vec<&String> = dtd_enum_keys.difference(&manifest_enum_keys).collect();
        failures.push(format!(
            "closed-enum key set mismatch (yorn enums excluded)\n  extra in manifest: {extra_m:?}\n  extra in DTD:      {extra_d:?}"
        ));
    }
    for (key, manifest_values) in &manifest.enums {
        let Some((elem, attr)) = key.split_once('.') else {
            failures.push(format!(
                "manifest enum key {key:?} is not in 'element.attr' form"
            ));
            continue;
        };
        let Some(dtd_attrs) = dtd.get(elem) else {
            continue;
        };
        let Some(dtd_attr) = dtd_attrs.iter().find(|a| a.name == attr) else {
            continue;
        };
        let m_set: BTreeSet<&str> = manifest_values.iter().map(String::as_str).collect();
        let d_set: BTreeSet<&str> = dtd_attr.enum_values.iter().map(String::as_str).collect();
        if m_set != d_set {
            let extra_m: Vec<&&str> = m_set.difference(&d_set).collect();
            let extra_d: Vec<&&str> = d_set.difference(&m_set).collect();
            failures.push(format!(
                "enum {key}: value mismatch\n  extra in manifest: {extra_m:?}\n  extra in DTD:      {extra_d:?}"
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "attribute manifest is out of sync with /System/Library/DTDs/sdef.dtd:\n\n{}\n\nUpdate tests/fixtures/attribute_manifest.toml AND the corresponding src/ struct fields, then re-run.",
            failures.join("\n\n")
        );
    }
}

#[test]
fn manifest_element_count_matches_known_elements() {
    // Even on non-macOS hosts, the count of modeled elements should match
    // the KNOWN_ELEMENTS slice in src/strict.rs (currently 25). This pins
    // a second tripwire so the manifest cannot silently drift away from
    // the strict-mode validator.
    let manifest = Manifest::load();
    assert_eq!(
        manifest.elements.len(),
        25,
        "manifest lists {} elements but strict-mode KNOWN_ELEMENTS has 25; \
         keep them in lock-step",
        manifest.elements.len(),
    );
}
