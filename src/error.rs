//! Public error type returned by parsing entry points.

use thiserror::Error;

/// Errors produced by [`crate::Dictionary`] parsing.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O failure while reading from a file or reader.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML deserialization failure (malformed input or unexpected shape).
    #[error("XML deserialization error: {0}")]
    Xml(#[from] quick_xml::DeError),
}
