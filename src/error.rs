//! Public error type returned by parsing entry points.

use thiserror::Error;

/// Errors produced by [`crate::Dictionary`] parsing.
///
/// # SemVer contract
///
/// The [`Error::Xml`] variant re-exports [`quick_xml::DeError`] as part of the
/// public API. Bumping the `quick-xml` major version is therefore a breaking
/// change for this crate: callers that pattern-match `Error::Xml(_)` and use
/// the wrapped value would need to update. This crate will bump its own major
/// version in lock-step when that happens.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O failure while reading from a file or reader.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML deserialization failure (malformed input or unexpected shape).
    #[error("XML deserialization error: {0}")]
    Xml(#[from] quick_xml::DeError),

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
}
