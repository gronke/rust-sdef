//! Suite-level type and member declarations: `<enumeration>`/`<enumerator>`,
//! `<record-type>`, `<value-type>`, `<property>`.
//!
//! These all *declare* something — an enumeration of constants, a structured
//! record type, an opaque value type, or a property member of a class or
//! record. They contrast with [`crate::TypeRef`], which is a *reference* to
//! such a declaration (or to one of the built-in primitives).

use serde::Deserialize;

use crate::metadata::{AccessGroup, Cocoa, Documentation, Synonym, Xref};
use crate::typeref::TypeRef;
use crate::yorn::{yorn, yorn_opt};

/// An `<enumeration>` of named constants.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Enumeration {
    /// Enumeration name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// `inline="N"` — display-compaction hint. Kept as a raw `String` to
    /// preserve the CDATA value (typically a decimal integer); callers may
    /// parse it as needed.
    #[serde(rename = "@inline", default)]
    pub inline: Option<String>,

    /// Optional `<cocoa>` implementation hint.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// `<enumerator>` children. At least one is required by the DTD.
    #[serde(rename = "enumerator", default)]
    pub enumerators: Vec<Enumerator>,

    /// Interleaved `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,

    /// Interleaved `<xref>` children.
    #[serde(rename = "xref", default)]
    pub xrefs: Vec<Xref>,
}

/// A single `<enumerator>` constant within an [`Enumeration`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Enumerator {
    /// Enumerator name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// Optional `<cocoa>` implementation hint.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default)]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}

/// A `<record-type>` — a structured value type composed of named properties,
/// distinct from a `<class>` (no behaviour, no identity).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct RecordType {
    /// Record-type name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// Optional plural form (`plural="…"`).
    #[serde(rename = "@plural", default)]
    pub plural: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default)]
    pub synonyms: Vec<Synonym>,

    /// `<property>` children declaring the fields of the record.
    #[serde(rename = "property", default)]
    pub properties: Vec<Property>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,

    /// `<xref>` children.
    #[serde(rename = "xref", default)]
    pub xrefs: Vec<Xref>,
}

/// A `<value-type>` — an opaque scalar type with no accessible properties or
/// elements, typically backed by a Cocoa class such as `NSColor`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct ValueType {
    /// Type name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// Optional plural form (`plural="…"`).
    #[serde(rename = "@plural", default)]
    pub plural: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default)]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,

    /// `<xref>` children.
    #[serde(rename = "xref", default)]
    pub xrefs: Vec<Xref>,
}

/// A `<property>` — a named, typed member of a `<class>`, `<class-extension>`,
/// or `<record-type>`. Properties model attributes and to-one relationships.
///
/// Lives here (alongside other declarations) rather than in `class.rs`
/// because record-type also declares properties; both consumers reach for the
/// same struct.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct Property {
    /// Property name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// Property value type (`type="…"`). Mutually exclusive with `<type>`
    /// child elements in well-formed sdefs.
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// `access="r|w|rw"` — defaults to `rw` per the man page; kept as
    /// `Option<String>` so the absent case is distinguishable from the
    /// explicit `"rw"`.
    #[serde(rename = "@access", default)]
    pub access: Option<String>,

    /// `in-properties="yes|no"` — whether this property appears in a
    /// `properties of …` record. Per the man page the DTD default is `yes`;
    /// `None` here means the attribute was omitted entirely.
    #[serde(rename = "@in-properties", default, deserialize_with = "yorn_opt")]
    pub in_properties: Option<bool>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children.
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// `<type>` child elements (list/union expressions).
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default)]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}
