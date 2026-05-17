//! Class hierarchy AST nodes: [`Class`], [`ClassExtension`], and their
//! members ([`Contents`], [`Element`], [`Accessor`], [`RespondsTo`]).
//!
//! Modelled after `<class>`, `<class-extension>`, `<contents>`, `<element>`,
//! `<accessor>`, and `<responds-to>` from the sdef DTD. Class-level
//! [`crate::Property`] declarations live in [`crate::decl`] because
//! `<record-type>` shares the same property concept.

use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::decl::Property;
use crate::metadata::{Access, AccessGroup, Cocoa, Documentation, Synonym, Xref};
use crate::typeref::TypeRef;
use crate::yorn;

/// A `<class>` declares a scriptable object type with properties, elements,
/// and the verbs it responds to.
///
/// The DTD permits the class-contents children (`contents`, `element`,
/// `property`, `responds-to`, `synonym`, `documentation`, `xref`) in any
/// order, and real-world sdefs (e.g. Xcode's) interleave them. We rely on
/// quick-xml's `overlapped-lists` cargo feature to deserialize each child
/// directly into its typed `Vec` field regardless of order. The same
/// applies to [`crate::Suite`] and [`ClassExtension`].
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Class {
    /// Class name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Optional plural form (`plural="…"`). Defaults to `<name>s` per the
    /// man page when omitted.
    #[serde(rename = "@plural", default, skip_serializing_if = "Option::is_none")]
    pub plural: Option<String>,

    /// Optional parent class name (`inherits="…"`). The AST stores this as a
    /// string; resolving the parent declaration is the caller's job.
    #[serde(rename = "@inherits", default, skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,

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

    /// Zero or more `<access-group>` entitlement children.
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,

    /// `<type>` children. The DTD permits `type*` here even though it's not
    /// strictly part of the AppleScript model; Cocoa Scripting uses it to
    /// give a class a value-type-style coercion target (e.g. `<class
    /// name="rich text">` has `<type type="text"/>`).
    #[serde(rename = "type", default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeRef>,

    /// `<property>` children, in document order.
    #[serde(rename = "property", default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,

    /// `<element>` children — to-many relationships, in document order.
    #[serde(rename = "element", default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,

    /// `<contents>` children — the implied container. Per the man page only
    /// one is meaningful, but the DTD allows the `<class-contents>` choice
    /// to repeat; we keep a `Vec` to surface malformed input rather than
    /// silently dropping extras.
    #[serde(rename = "contents", default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Contents>,

    /// `<responds-to>` children declaring which verbs the class handles.
    #[serde(rename = "responds-to", default, skip_serializing_if = "Vec::is_empty")]
    pub responds_to: Vec<RespondsTo>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,

    /// `<xref>` children.
    #[serde(rename = "xref", default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Xref>,
}

/// A `<class-extension>` adds properties, elements, or `responds-to`
/// declarations to a class declared elsewhere.
///
/// Like [`Class`], relies on quick-xml's `overlapped-lists` feature so that
/// class-contents children can appear in any order.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ClassExtension {
    /// `extends="…"` — name of the class being extended (required).
    #[serde(rename = "@extends")]
    pub extends: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// `title="…"` — display title for the extension (added in OS X 10.10).
    #[serde(rename = "@title", default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

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

    /// Zero or more `<access-group>` entitlement children.
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,

    /// `<property>` children added by this extension, in document order.
    #[serde(rename = "property", default, skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<Property>,

    /// `<element>` children added by this extension, in document order.
    #[serde(rename = "element", default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<Element>,

    /// `<contents>` children added by this extension.
    #[serde(rename = "contents", default, skip_serializing_if = "Vec::is_empty")]
    pub contents: Vec<Contents>,

    /// `<responds-to>` children added by this extension.
    #[serde(rename = "responds-to", default, skip_serializing_if = "Vec::is_empty")]
    pub responds_to: Vec<RespondsTo>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,

    /// `<xref>` children.
    #[serde(rename = "xref", default, skip_serializing_if = "Vec::is_empty")]
    pub xrefs: Vec<Xref>,
}

/// A `<contents>` element — the implied container of a class.
///
/// Lets AppleScript treat `word 1 of document 1` as shorthand for `word 1
/// of text of document 1`. Most attributes are optional and default per the
/// man page: `name="contents"`, `code="pcnt"`.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Contents {
    /// `name="…"` — defaults to `"contents"` per the man page.
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// `code="…"` — defaults to `"pcnt"` per the man page.
    #[serde(rename = "@code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// `type="…"` — value type of the container. Mutually exclusive with
    /// `<type>` children in well-formed sdefs.
    #[serde(rename = "@type", default, skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,

    /// `access="r|w|rw"` — defaults to `rw` per the man page.
    #[serde(rename = "@access", default, skip_serializing_if = "Option::is_none")]
    pub access: Option<Access>,

    /// `in-properties="yes|no"` — per the man page the DTD default is `yes`,
    /// so use `unwrap_or(true)` to apply the default when absent.
    #[serde(
        rename = "@in-properties",
        default,
        deserialize_with = "yorn::de_opt",
        serialize_with = "yorn::ser_opt",
        skip_serializing_if = "Option::is_none"
    )]
    pub in_properties: Option<bool>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

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

    /// Zero or more `<access-group>` entitlement children.
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,

    /// `<type>` child elements.
    #[serde(rename = "type", default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TypeRef>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default, skip_serializing_if = "Vec::is_empty")]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}

/// An `<element>` — a to-many relationship from a class to instances of
/// another class.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Element {
    /// `type="…"` — class name of the contained objects (required).
    #[serde(rename = "@type")]
    pub ty: String,

    /// `access="r|w|rw"` — defaults to `rw` per the man page.
    #[serde(rename = "@access", default, skip_serializing_if = "Option::is_none")]
    pub access: Option<Access>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

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

    /// Zero or more `<access-group>` entitlement children.
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,

    /// `<accessor>` children declaring supported access styles.
    #[serde(rename = "accessor", default, skip_serializing_if = "Vec::is_empty")]
    pub accessors: Vec<Accessor>,

    /// `<documentation>` children.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}

/// An `<accessor>` describing how scripts may reach an element.
///
/// Used by aete-based dictionaries; Cocoa Scripting derives access styles
/// from properties and largely ignores explicit `<accessor>` declarations.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Accessor {
    /// `style="…"` — one of the six DTD-declared `accessor-type` values.
    /// Unknown values deserialize to [`AccessorStyle::Other`] so lenient
    /// parsing keeps working; strict-mode parsing rejects them via the
    /// element-name pre-pass plus the
    /// `tests/attribute_conformance` test that keeps this enum in sync
    /// with the DTD.
    #[serde(rename = "@style")]
    pub style: AccessorStyle,
}

/// The closed set of `<accessor style="…">` values declared by the sdef
/// DTD's `accessor-type` entity: `(index | name | id | range | relative |
/// test)`.
///
/// Unknown values deserialize to [`AccessorStyle::Other`] for lenient mode;
/// the variant list is kept in lock-step with the DTD via the
/// `attribute_conformance` integration test.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccessorStyle {
    /// `style="index"` — by 1-based ordinal.
    Index,
    /// `style="name"` — by `name` attribute.
    Name,
    /// `style="id"` — by `id` attribute.
    Id,
    /// `style="range"` — by inclusive range of two specifiers.
    Range,
    /// `style="relative"` — relative to another specifier (e.g. `before`/`after`).
    Relative,
    /// `style="test"` — by predicate (`every X where …`).
    Test,
    /// Unrecognised `style` value, preserved verbatim for lenient parsing.
    Other(String),
}

impl AccessorStyle {
    /// Returns the canonical DTD string for this variant, or the wrapped
    /// value for [`AccessorStyle::Other`]. Mirrors the textual form the
    /// parser accepts.
    pub fn as_str(&self) -> &str {
        match self {
            AccessorStyle::Index => "index",
            AccessorStyle::Name => "name",
            AccessorStyle::Id => "id",
            AccessorStyle::Range => "range",
            AccessorStyle::Relative => "relative",
            AccessorStyle::Test => "test",
            AccessorStyle::Other(s) => s,
        }
    }
}

impl fmt::Display for AccessorStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AccessorStyle {
    type Err = Infallible;

    /// Infallible: unknown values map to [`AccessorStyle::Other`].
    /// Strict-mode parsing rejects out-of-range styles at the document
    /// level via [`crate::Dictionary::from_str_strict`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "index" => AccessorStyle::Index,
            "name" => AccessorStyle::Name,
            "id" => AccessorStyle::Id,
            "range" => AccessorStyle::Range,
            "relative" => AccessorStyle::Relative,
            "test" => AccessorStyle::Test,
            _ => AccessorStyle::Other(s.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for AccessorStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "index" => AccessorStyle::Index,
            "name" => AccessorStyle::Name,
            "id" => AccessorStyle::Id,
            "range" => AccessorStyle::Range,
            "relative" => AccessorStyle::Relative,
            "test" => AccessorStyle::Test,
            _ => AccessorStyle::Other(raw),
        })
    }
}

impl Serialize for AccessorStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// A `<responds-to>` declaration mapping a verb to a class's
/// implementation.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RespondsTo {
    /// `command="…"` — the verb name or id this class handles.
    ///
    /// Required since OS X 10.5; older sdefs may use the deprecated `name`
    /// attribute instead. We model `command` as `Option<String>` so legacy
    /// dictionaries deserialize cleanly; consumers should treat
    /// `command.or(name)` as the canonical reference.
    #[serde(rename = "@command", default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// `name="…"` — pre-OS X 10.5 alias for `command`, still accepted by
    /// the DTD for backward compatibility.
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional `<cocoa>` implementation hint child (e.g. method selector
    /// for custom verb handlers).
    #[serde(rename = "cocoa", default, skip_serializing_if = "Option::is_none")]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children.
    #[serde(
        rename = "access-group",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub access_groups: Vec<AccessGroup>,
}

impl RespondsTo {
    /// Returns the canonical command reference, preferring `command` (the
    /// modern attribute introduced in OS X 10.5) and falling back to the
    /// pre-10.5 `name` alias. Returns `None` only when both attributes are
    /// absent — a malformed `<responds-to>` per the DTD.
    pub fn resolved_command(&self) -> Option<&str> {
        self.command.as_deref().or(self.name.as_deref())
    }
}
