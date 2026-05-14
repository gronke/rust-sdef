//! Command parameter AST nodes.
//!
//! Modelled after `<parameter>`, `<direct-parameter>`, and `<result>` from the
//! sdef DTD. The DTD's `(yes | no)` entity is converted to `bool` via the
//! [`crate::yorn`] helper.

use serde::Deserialize;

use crate::yorn::yorn;

/// A `<parameter>` of a `<command>`.
#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    /// Human-readable parameter name (`name="…"`), e.g. `"from date"`.
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character parameter code (`code="…"`), e.g. `"frdt"`.
    #[serde(rename = "@code")]
    pub code: String,

    /// Parameter value type as documented by the DTD (`type="…"`); typically
    /// `text`, `real`, `integer`, `boolean`, `any`, or a `<class>` name.
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// `optional="yes"` flag, defaults to `false`.
    #[serde(rename = "@optional", default, deserialize_with = "yorn")]
    pub optional: bool,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,
}

/// The un-named first argument of a command (`<direct-parameter>`).
///
/// Carries the same attributes as a regular parameter except for `name` —
/// direct parameters are positional in AppleScript syntax.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectParameter {
    /// Value type (`type="…"`).
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// `optional="yes"` flag.
    #[serde(rename = "@optional", default, deserialize_with = "yorn")]
    pub optional: bool,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,
}

/// A `<result>` element describing a command's return value.
///
/// Named with a trailing underscore to avoid clashing with the prelude's
/// `Result`.
#[derive(Debug, Clone, Deserialize)]
pub struct Result_ {
    /// Result value type (`type="…"`).
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,
}
