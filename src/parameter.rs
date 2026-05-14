//! Command parameter AST nodes.
//!
//! Modelled after `<parameter>`, `<direct-parameter>`, and `<result>` from the
//! sdef DTD. The DTD's `(yes | no)` entity is converted to `bool` via the
//! [`crate::yorn`] helper.

use serde::Deserialize;

use crate::metadata::{Cocoa, Documentation};
use crate::typeref::TypeRef;
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
    /// Mutually exclusive with [`Self::types`] in well-formed sdefs.
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// `optional="yes"` flag, defaults to `false`.
    #[serde(rename = "@optional", default, deserialize_with = "yorn")]
    pub optional: bool,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// `requires-access="r|w|rw"` — sandbox-access requirement for this
    /// parameter's value. `None` when the attribute is absent.
    #[serde(rename = "@requires-access", default)]
    pub requires_access: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// `<type>` child elements. Used when the parameter takes a list type,
    /// a union of types, or when the inline `type` attribute is omitted in
    /// favour of richer markup.
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks (since OS X 10.10 may appear inline
    /// alongside parameters within a command).
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
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

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// `requires-access="r|w|rw"` — sandbox-access requirement.
    #[serde(rename = "@requires-access", default)]
    pub requires_access: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `<type>` child elements (list/union expressions).
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
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

    /// `<type>` child elements (list/union expressions).
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}
