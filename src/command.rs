//! Verb-family AST nodes: [`Command`], [`Event`], and the parameter/result
//! types they share.
//!
//! Modelled after the `<command>`, `<event>`, `<parameter>`,
//! `<direct-parameter>`, and `<result>` elements defined in the sdef DTD.
//! Commands are script→object verbs; events are system→object notifications;
//! both share the same parameter/result vocabulary.

use serde::{Deserialize, Serialize};

use crate::metadata::{Access, AccessGroup, Cocoa, Documentation, Synonym, Xref};
use crate::typeref::TypeRef;
use crate::yorn;

/// A `<command>` — a verb the application supports via Apple Events,
/// invoked from a script.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Command {
    /// Human-readable command name (`name="…"`), e.g. `"export transactions"`.
    #[serde(rename = "@name")]
    pub name: String,

    /// Eight-character Apple Event code (`code="…"`), e.g. `"MONYexpt"`.
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier for cross-references via
    /// `<xref>` or `<responds-to>`.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default, skip_serializing_if = "Option::is_none")]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children (since OS X 10.8).
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,

    /// Zero or more `<synonym>` children — alternate names/codes.
    #[serde(rename = "synonym", default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` child blocks.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,

    /// Optional `<direct-parameter>` (the un-named first argument).
    ///
    /// Declared before [`Self::parameters`] to match the DTD content model:
    /// `((direct-parameter , (parameter | documentation)*) | …)`. Emission
    /// order matters — `xmllint --dtdvalid` rejects documents that emit
    /// `<parameter>` before `<direct-parameter>`.
    #[serde(
        rename = "direct-parameter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub direct_parameter: Option<DirectParameter>,

    /// `<parameter>` children, in document order.
    #[serde(rename = "parameter", default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,

    /// Optional `<result>` element describing the command's return value.
    #[serde(rename = "result", default, skip_serializing_if = "Option::is_none")]
    pub result: Option<CommandResult>,

    /// Zero or more `<xref>` cross-reference children (since OS X 10.5).
    #[serde(rename = "xref", default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Xref>,
}

/// An `<event>` — a verb the application receives from the system, e.g.
/// `"opened document"` for an `aevtodoc` Apple Event.
///
/// Structurally identical to [`Command`] aside from the absence of
/// `<access-group>` children: events are inbound notifications, so the
/// caller-side entitlement model doesn't apply.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Event {
    /// Human-readable event name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Eight-character Apple Event code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default, skip_serializing_if = "Option::is_none")]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<synonym>` children.
    #[serde(rename = "synonym", default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` child blocks.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,

    /// Optional `<direct-parameter>`.
    ///
    /// Declared before [`Self::parameters`] to match the DTD content model;
    /// see the corresponding field on [`Command`] for the full rationale.
    #[serde(
        rename = "direct-parameter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub direct_parameter: Option<DirectParameter>,

    /// `<parameter>` children.
    #[serde(rename = "parameter", default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,

    /// Optional `<result>`.
    #[serde(rename = "result", default, skip_serializing_if = "Option::is_none")]
    pub result: Option<CommandResult>,

    /// Zero or more `<xref>` cross-reference children.
    #[serde(rename = "xref", default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Xref>,
}

/// A `<parameter>` of a `<command>` or `<event>`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
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
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,

    /// `optional="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@optional",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub optional: bool,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// `requires-access="r|w|rw"` — sandbox-access requirement for this
    /// parameter's value. `None` when the attribute is absent.
    #[serde(
        rename = "@requires-access",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub requires_access: Option<Access>,

    /// Optional human description (`description="…"`).
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default, skip_serializing_if = "Option::is_none")]
    pub cocoa: Option<Cocoa>,

    /// `<type>` child elements. Used when the parameter takes a list type,
    /// a union of types, or when the inline `type` attribute is omitted in
    /// favour of richer markup.
    #[serde(rename = "type", default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks (since OS X 10.10 may appear inline
    /// alongside parameters within a command).
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}

/// The un-named first argument of a command (`<direct-parameter>`).
///
/// Carries the same attributes as a regular parameter except for `name` —
/// direct parameters are positional in AppleScript syntax.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DirectParameter {
    /// Value type (`type="…"`).
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,

    /// `optional="yes"` flag.
    #[serde(
        rename = "@optional",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub optional: bool,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// `requires-access="r|w|rw"` — sandbox-access requirement.
    #[serde(
        rename = "@requires-access",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub requires_access: Option<Access>,

    /// Optional human description (`description="…"`).
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// `<type>` child elements (list/union expressions).
    #[serde(rename = "type", default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks (since OS X 10.10 may appear inline
    /// inside a direct-parameter).
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}

/// A `<result>` element describing a command's return value.
///
/// Named `CommandResult` to avoid clashing with the prelude's `Result` while
/// staying readable in API surface (versus the earlier `Result_`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct CommandResult {
    /// Result value type (`type="…"`).
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(
        rename = "@description",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub description: Option<String>,

    /// `<type>` child elements (list/union expressions).
    #[serde(rename = "type", default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeRef>,

    /// `<documentation>` child blocks (since OS X 10.10 may appear inline
    /// inside a result).
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}
