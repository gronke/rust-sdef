//! Public error type returned by parsing and emission entry points.

use thiserror::Error;

/// Errors produced by [`crate::Dictionary`] parsing and serialisation.
///
/// # SemVer contract
///
/// The [`Error::Xml`] and [`Error::Ser`] variants re-export
/// [`quick_xml::DeError`] and [`quick_xml::SeError`] as part of the public
/// API. Bumping the `quick-xml` major version is therefore a breaking
/// change for this crate: callers that pattern-match these variants and
/// use the wrapped value would need to update. This crate will bump its
/// own major version in lock-step when that happens.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O failure while reading from a file or reader, or while writing
    /// to a file or writer.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML deserialization failure (malformed input or unexpected shape).
    #[error("XML deserialization error: {0}")]
    Xml(#[from] quick_xml::DeError),

    /// XML serialisation failure produced by
    /// [`crate::Dictionary::to_xml_string`] and friends. Indicates a
    /// structural problem the emitter could not represent (e.g., a
    /// `String` field containing characters that would produce malformed
    /// XML).
    #[error("XML serialisation error: {0}")]
    Ser(#[from] quick_xml::SeError),

    /// Strict-mode validation encountered an element name that this crate
    /// does not model. The payload is the local name of the offending
    /// element. Indicates DTD drift, vendor-specific extensions, or an
    /// element this crate has not yet added.
    #[error("unknown XML element <{name}> rejected by strict-mode validation")]
    UnknownElement {
        /// The local name of the unknown element.
        name: String,
    },

    /// Strict-mode validation encountered an `<xi:include>` directive.
    /// This crate's deserializer does not resolve XInclude; the document
    /// would parse with the include's content silently ignored, so strict
    /// mode rejects it instead.
    #[error("<xi:include> is not supported by this parser; resolve includes before parsing")]
    XIncludeUnsupported,

    /// Strict-mode validation encountered an attribute whose value is
    /// outside the closed set declared by the DTD. For example, the DTD
    /// declares `<accessor style="…">` as one of
    /// `(index | name | id | range | relative | test)`, and any other
    /// string (`<accessor style="weird"/>`) trips this error in strict mode.
    /// Lenient parsing tolerates the unknown value via the `Other(String)`
    /// variant on the corresponding typed enum.
    #[error("<{element} {attribute}={value:?}> is outside the DTD's closed enumeration")]
    UnknownAttributeValue {
        /// The local name of the element carrying the attribute.
        element: String,
        /// The local name of the offending attribute.
        attribute: String,
        /// The actual value found in the source XML.
        value: String,
    },
}
