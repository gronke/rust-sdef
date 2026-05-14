//! Class hierarchy AST nodes: [`Class`], [`ClassExtension`], and their
//! members ([`Contents`], [`Element`], [`Accessor`], [`RespondsTo`]).
//!
//! Modelled after `<class>`, `<class-extension>`, `<contents>`, `<element>`,
//! `<accessor>`, and `<responds-to>` from the sdef DTD. Class-level
//! [`crate::Property`] declarations live in [`crate::decl`] because
//! `<record-type>` shares the same property concept.

use serde::Deserialize;

use crate::decl::Property;
use crate::metadata::{AccessGroup, Cocoa, Documentation, Synonym, Xref};
use crate::typeref::TypeRef;
use crate::yorn::yorn;

/// A `<class>` declares a scriptable object type with properties, elements,
/// and the verbs it responds to.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Class {
    /// Class name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// Optional plural form (`plural="…"`). Defaults to `<name>s` per the
    /// man page when omitted.
    #[serde(rename = "@plural", default)]
    pub plural: Option<String>,

    /// Optional parent class name (`inherits="…"`). The AST stores this as a
    /// string; resolving the parent declaration is the caller's job.
    #[serde(rename = "@inherits", default)]
    pub inherits: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children.
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// `<type>` children. The DTD permits `type*` here even though it's not
    /// strictly part of the AppleScript model; Cocoa Scripting uses it to
    /// give a class a value-type-style coercion target (e.g. `<class
    /// name="rich text">` has `<type type="text"/>`).
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<property>` children.
    #[serde(rename = "property", default)]
    pub properties: Vec<Property>,

    /// `<element>` children — to-many relationships.
    #[serde(rename = "element", default)]
    pub elements: Vec<Element>,

    /// `<contents>` children — the implied container. Per the man page only
    /// one is meaningful, but the DTD allows the `<class-contents>` choice
    /// to repeat; we keep a `Vec` to surface malformed input rather than
    /// silently dropping extras.
    #[serde(rename = "contents", default)]
    pub contents: Vec<Contents>,

    /// `<responds-to>` children declaring which verbs the class handles.
    #[serde(rename = "responds-to", default)]
    pub responds_to: Vec<RespondsTo>,

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

/// A `<class-extension>` adds properties, elements, or `responds-to`
/// declarations to a class declared elsewhere.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct ClassExtension {
    /// `extends="…"` — name of the class being extended (required).
    #[serde(rename = "@extends")]
    pub extends: String,

    /// `id="…"` — optional unique identifier.
    #[serde(rename = "@id", default)]
    pub id: Option<String>,

    /// `title="…"` — display title for the extension (added in OS X 10.10).
    #[serde(rename = "@title", default)]
    pub title: Option<String>,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children.
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// `<property>` children added by this extension.
    #[serde(rename = "property", default)]
    pub properties: Vec<Property>,

    /// `<element>` children added by this extension.
    #[serde(rename = "element", default)]
    pub elements: Vec<Element>,

    /// `<contents>` children added by this extension.
    #[serde(rename = "contents", default)]
    pub contents: Vec<Contents>,

    /// `<responds-to>` children added by this extension.
    #[serde(rename = "responds-to", default)]
    pub responds_to: Vec<RespondsTo>,

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

/// A `<contents>` element — the implied container of a class.
///
/// Lets AppleScript treat `word 1 of document 1` as shorthand for `word 1
/// of text of document 1`. Most attributes are optional and default per the
/// man page: `name="contents"`, `code="pcnt"`.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Contents {
    /// `name="…"` — defaults to `"contents"` per the man page.
    #[serde(rename = "@name", default)]
    pub name: Option<String>,

    /// `code="…"` — defaults to `"pcnt"` per the man page.
    #[serde(rename = "@code", default)]
    pub code: Option<String>,

    /// `type="…"` — value type of the container. Mutually exclusive with
    /// `<type>` children in well-formed sdefs.
    #[serde(rename = "@type", default)]
    pub ty: Option<String>,

    /// `access="r|w|rw"` — defaults to `rw` per the man page.
    #[serde(rename = "@access", default)]
    pub access: Option<String>,

    /// `in-properties="yes|no"` — per the man page the DTD default is `yes`.
    #[serde(rename = "@in-properties", default)]
    pub in_properties: Option<String>,

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

    /// `<type>` child elements.
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,

    /// `<synonym>` children.
    #[serde(rename = "synonym", default)]
    pub synonyms: Vec<Synonym>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}

/// An `<element>` — a to-many relationship from a class to instances of
/// another class.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Element {
    /// `type="…"` — class name of the contained objects (required).
    #[serde(rename = "@type")]
    pub ty: String,

    /// `access="r|w|rw"` — defaults to `rw` per the man page.
    #[serde(rename = "@access", default)]
    pub access: Option<String>,

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

    /// `<accessor>` children declaring supported access styles.
    #[serde(rename = "accessor", default)]
    pub accessors: Vec<Accessor>,

    /// `<documentation>` children.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}

/// An `<accessor>` describing how scripts may reach an element.
///
/// Used by aete-based dictionaries; Cocoa Scripting derives access styles
/// from properties and largely ignores explicit `<accessor>` declarations.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Accessor {
    /// `style="…"` — one of `index`, `name`, `id`, `range`, `relative`,
    /// `test`. Kept as a raw `String`; the parser does not enforce the
    /// enum.
    #[serde(rename = "@style")]
    pub style: String,
}

/// A `<responds-to>` declaration mapping a verb to a class's
/// implementation.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct RespondsTo {
    /// `command="…"` — the verb name or id this class handles.
    ///
    /// Required since OS X 10.5; older sdefs may use the deprecated `name`
    /// attribute instead. We model `command` as `Option<String>` so legacy
    /// dictionaries deserialize cleanly; consumers should treat
    /// `command.or(name)` as the canonical reference.
    #[serde(rename = "@command", default)]
    pub command: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// `name="…"` — pre-OS X 10.5 alias for `command`, still accepted by
    /// the DTD for backward compatibility.
    #[serde(rename = "@name", default)]
    pub name: Option<String>,

    /// Optional `<cocoa>` implementation hint child (e.g. method selector
    /// for custom verb handlers).
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children.
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,
}
