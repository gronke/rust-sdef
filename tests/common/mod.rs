//! Shared helpers for the conformance integration tests.
//!
//! Used by `tests/attribute_conformance.rs` and the conformance matrix
//! generator. Files under `tests/common/` are not auto-discovered as
//! separate test binaries — each consuming test file must declare
//! `mod common;` to bring this in.
//!
//! DTD parsing here is delegated to libxml2 (via the `libxml` dev-dependency)
//! rather than implemented in-tree. Apple's sdef DTD relies on parameter
//! entities (`%common.attrib;`, `%yorn;`, `%rw;`, `%accessor-type;`) and
//! `%`-expansion that a hand-rolled parser had to track by hand; libxml2
//! is the canonical implementation and handles all of it transparently.

#![allow(dead_code)] // not every consumer uses every helper

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::sync::Once;

use serde::{Deserialize, Serialize};

/// Path to the system DTD on macOS.
pub const DTD_PATH: &str = "/System/Library/DTDs/sdef.dtd";

/// Returns `Some(path)` to the system DTD if present, `None` on hosts where
/// it is absent (non-macOS CI runners). Callers should skip-and-return on
/// `None` so the suite stays green on Ubuntu.
pub fn system_dtd_path() -> Option<&'static Path> {
    let p = Path::new(DTD_PATH);
    if p.exists() { Some(p) } else { None }
}

/// Path to the curated attribute manifest checked in under `tests/fixtures/`.
pub const MANIFEST_PATH: &str = "tests/fixtures/attribute_manifest.toml";

/// Hand-curated record of every element + attribute this crate models.
#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "element")]
    pub elements: Vec<ElementSpec>,
    pub enums: BTreeMap<String, Vec<String>>,
}

/// Per-element manifest entry — attribute names and which the DTD marks
/// `#REQUIRED`.
#[derive(Debug, Deserialize)]
pub struct ElementSpec {
    pub name: String,
    pub attributes: Vec<String>,
    pub required: Vec<String>,
}

impl Manifest {
    /// Load and parse `tests/fixtures/attribute_manifest.toml`.
    pub fn load() -> Self {
        let text = std::fs::read_to_string(MANIFEST_PATH).expect("read attribute_manifest.toml");
        toml::from_str(&text).expect("parse attribute_manifest.toml")
    }
}

/// One DTD-declared attribute — name, whether it's required, and (for closed
/// enumerations) the allowed value set.
#[derive(Debug, Default, Clone)]
pub struct DtdAttr {
    pub name: String,
    pub required: bool,
    pub enum_values: Vec<String>,
}

/// Parse a DTD file via libxml2 into a map of `element_name -> attributes`.
/// Elements with no `<!ATTLIST>` (e.g. `<documentation>`, `<html>`) appear
/// with an empty attribute list.
///
/// This wraps libxml2's full DTD parser through the `libxml::bindings`
/// raw FFI surface. The high-level `libxml` Rust API exposes XML
/// parsing but not Dtd walking, so we drop one level down to call
/// `xmlIOParseDTD` + `xmlHashScan` directly. Parameter-entity expansion
/// (`%common.attrib;`, `%yorn;`, `%rw;`, `%accessor-type;`) happens
/// transparently inside libxml2 — no glue code needed at this layer.
///
/// XML-namespace plumbing attributes (`xmlns:xi`, `xml:base`) that
/// `%common.attrib;` drags in are filtered out: they're not sdef
/// terminology.
///
/// Dev-only path: the published `sdef` crate has no dependency on libxml.
pub fn parse_dtd(path: &Path) -> BTreeMap<String, Vec<DtdAttr>> {
    use libxml::bindings::{
        xmlCharEncoding_XML_CHAR_ENCODING_UTF8, xmlFreeDtd, xmlHashScan, xmlHashTablePtr,
        xmlIOParseDTD, xmlInitParser, xmlParserInputBufferCreateFilename,
    };

    let mut result: BTreeMap<String, Vec<DtdAttr>> = BTreeMap::new();

    let Ok(c_path) = CString::new(path.to_string_lossy().as_bytes()) else {
        return result;
    };

    // SAFETY: xmlInitParser is safe to call multiple times; guarded by Once
    // for thread-safety. xmlParserInputBufferCreateFilename returns NULL on
    // I/O error and we check before use. xmlIOParseDTD takes ownership of
    // the input buffer (frees it internally) and returns NULL on parse error.
    // xmlHashScan iterates synchronously inside this scope so the `ScanAcc`
    // borrow remains live for the duration. xmlFreeDtd is called exactly
    // once on success.
    unsafe {
        INIT_PARSER.call_once(|| xmlInitParser());

        let buf = xmlParserInputBufferCreateFilename(
            c_path.as_ptr(),
            xmlCharEncoding_XML_CHAR_ENCODING_UTF8,
        );
        if buf.is_null() {
            return result;
        }

        let dtd = xmlIOParseDTD(
            std::ptr::null_mut(),
            buf,
            xmlCharEncoding_XML_CHAR_ENCODING_UTF8,
        );
        if dtd.is_null() {
            return result;
        }

        let elements = (*dtd).elements as xmlHashTablePtr;
        if !elements.is_null() {
            let mut acc = ScanAcc {
                result: &mut result,
            };
            xmlHashScan(
                elements,
                Some(element_visitor),
                (&mut acc as *mut ScanAcc).cast::<c_void>(),
            );
        }

        xmlFreeDtd(dtd);
    }

    result
}

static INIT_PARSER: Once = Once::new();

/// Accumulator passed through the libxml2 hash-scan callback as `*mut c_void`.
struct ScanAcc<'a> {
    result: &'a mut BTreeMap<String, Vec<DtdAttr>>,
}

/// libxml2 hash-scan callback. Invoked once per element declaration in the
/// DTD. Walks the element's attribute linked list and pushes a `DtdAttr`
/// for each one (excluding `xmlns:xi` / `xml:base` plumbing).
///
/// # Safety
///
/// Invoked synchronously by libxml2 from `xmlHashScan`. `payload` is the
/// `xmlElementPtr` stored for `name`. `data` is the `&mut ScanAcc` we passed
/// when calling `xmlHashScan`. All three are guaranteed non-null by libxml2
/// for entries in the elements hash table; defensive null-checks are kept
/// for belt-and-braces.
unsafe extern "C" fn element_visitor(payload: *mut c_void, data: *mut c_void, _name: *const u8) {
    use libxml::bindings::{
        _xmlElement, xmlAttributeDefault_XML_ATTRIBUTE_REQUIRED,
        xmlAttributeType_XML_ATTRIBUTE_ENUMERATION,
    };

    if payload.is_null() || data.is_null() {
        return;
    }

    // SAFETY: payload is an xmlElementPtr supplied by libxml2; data is the
    // pointer we passed to xmlHashScan from a live mutable borrow that
    // outlives this call.
    let elem = unsafe { &*payload.cast::<_xmlElement>() };
    let acc = unsafe { &mut *data.cast::<ScanAcc<'_>>() };

    let elem_name = unsafe { c_str_to_string(elem.name) };

    let mut attrs: Vec<DtdAttr> = Vec::new();
    let mut attr_ptr = elem.attributes;
    while !attr_ptr.is_null() {
        // SAFETY: attr_ptr is non-null and points to an xmlAttribute owned
        // by the DTD; the linked list is terminated by NULL.
        let attr = unsafe { &*attr_ptr };
        let attr_name = unsafe { c_str_to_string(attr.name) };
        let attr_prefix = unsafe { c_str_to_string(attr.prefix) };

        // %common.attrib; expands to include xmlns:xi and xml:base; libxml2
        // surfaces them with `prefix` set to "xmlns" / "xml" and `name` to
        // the local part. They are XML-namespace plumbing, not sdef
        // terminology, and the attribute manifest doesn't list them.
        let is_xml_plumbing = matches!(attr_prefix.as_str(), "xmlns" | "xml");
        if !is_xml_plumbing {
            let required = attr.def == xmlAttributeDefault_XML_ATTRIBUTE_REQUIRED;
            let mut enum_values: Vec<String> = Vec::new();
            if attr.atype == xmlAttributeType_XML_ATTRIBUTE_ENUMERATION {
                let mut node = attr.tree;
                while !node.is_null() {
                    // SAFETY: node points to an xmlEnumeration owned by the
                    // attribute; the linked list is terminated by NULL.
                    let n = unsafe { &*node };
                    enum_values.push(unsafe { c_str_to_string(n.name) });
                    node = n.next;
                }
            }
            attrs.push(DtdAttr {
                name: attr_name,
                required,
                enum_values,
            });
        }
        attr_ptr = attr.nexth;
    }

    acc.result.insert(elem_name, attrs);
}

/// Decode a `*const xmlChar` (null-terminated, UTF-8) into an owned String.
///
/// # Safety
///
/// Caller must ensure `p` either is null or points to a NUL-terminated C
/// string with a lifetime at least as long as the call.
unsafe fn c_str_to_string(p: *const u8) -> String {
    if p.is_null() {
        return String::new();
    }
    // SAFETY: p is non-null and points to a NUL-terminated string per caller.
    unsafe { CStr::from_ptr(p.cast::<c_char>()) }
        .to_string_lossy()
        .into_owned()
}

/// Walk an sdef XML document once and collect every `(local element name,
/// local attribute name)` pair that appears. Used by `corpus_smoke` to
/// build a coverage map of which DTD constructs are exercised by real-world
/// fixtures, and consumed by the conformance-matrix generator.
///
/// XML namespaces are stripped from both element and attribute names so the
/// resulting map keys directly match `attribute_manifest.toml` entries.
pub fn scan_xml_coverage(xml: &str) -> BTreeMap<String, BTreeSet<String>> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let mut result: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) | Err(_) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = String::from_utf8_lossy(e.local_name().as_ref()).into_owned();
                let entry = result.entry(local).or_default();
                for attr in e.attributes().flatten() {
                    let key = attr.key;
                    if key.prefix().is_some() {
                        continue; // ignore xmlns:*, xml:base, etc.
                    }
                    let attr_local =
                        String::from_utf8_lossy(key.local_name().as_ref()).into_owned();
                    if !attr_local.is_empty() {
                        entry.insert(attr_local);
                    }
                }
            }
            _ => {}
        }
    }
    result
}

/// JSON-serialisable shape emitted by `corpus_smoke` to
/// `target/corpus_coverage.json` and read back by the conformance matrix
/// generator. Stored relative to repo root so CI artifacts and local
/// invocations agree on the path.
pub const CORPUS_COVERAGE_PATH: &str = "target/corpus_coverage.json";

/// Top-level shape of `target/corpus_coverage.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusCoverage {
    /// Per-fixture coverage map: fixture path -> element -> attribute set.
    pub fixtures: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    /// Aggregate union across every fixture.
    pub aggregate: BTreeMap<String, BTreeSet<String>>,
    /// Total fixtures scanned. Convenience for matrix rendering.
    pub total_fixtures: usize,
}

impl CorpusCoverage {
    /// Build an aggregate map by unioning every fixture's coverage.
    pub fn from_fixtures(fixtures: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>) -> Self {
        let mut aggregate: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for per_fixture in fixtures.values() {
            for (elem, attrs) in per_fixture {
                let entry = aggregate.entry(elem.clone()).or_default();
                for a in attrs {
                    entry.insert(a.clone());
                }
            }
        }
        let total_fixtures = fixtures.len();
        Self {
            fixtures,
            aggregate,
            total_fixtures,
        }
    }

    /// Persist to `target/corpus_coverage.json` (creating `target/` if needed).
    pub fn write_default(&self) -> std::io::Result<()> {
        if let Some(parent) = Path::new(CORPUS_COVERAGE_PATH).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self).expect("serialize CorpusCoverage");
        std::fs::write(CORPUS_COVERAGE_PATH, json)
    }

    /// Load `target/corpus_coverage.json` if it exists. Returns `None` when
    /// the file is absent so the matrix renderer can fall back to "—".
    pub fn load_default() -> Option<Self> {
        let path = Path::new(CORPUS_COVERAGE_PATH);
        if !path.exists() {
            return None;
        }
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }
}
