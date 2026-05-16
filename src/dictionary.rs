//! Top-level structural AST nodes: [`Dictionary`] and [`Suite`].
//!
//! Modelled after the `<dictionary>` and `<suite>` elements defined in
//! `/System/Library/DTDs/sdef.dtd`. Verb-family types ([`crate::Command`],
//! [`crate::Event`]) live in [`crate::command`]; type/member declarations
//! ([`crate::Enumeration`], [`crate::RecordType`], [`crate::ValueType`],
//! [`crate::Property`]) live in [`crate::decl`].

use serde::{Deserialize, Serialize};

use crate::class::{Class, ClassExtension};
use crate::command::{Command, Event};
use crate::decl::{Enumeration, RecordType, ValueType};
use crate::metadata::{AccessGroup, Cocoa, Documentation};
use crate::yorn;

/// The root `<dictionary>` element of an sdef document.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename = "dictionary")]
#[non_exhaustive]
pub struct Dictionary {
    /// Optional human-readable title attribute (`title="…"`).
    #[serde(rename = "@title", default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// All `<suite>` children. The DTD requires at least one suite.
    #[serde(rename = "suite", default, skip_serializing_if = "Vec::is_empty")]
    pub suites: Vec<Suite>,

    /// Optional root-level `<documentation>` siblings of `<suite>`. The DTD's
    /// content model for `<dictionary>` is `(documentation*, suite+)`, so a
    /// well-formed sdef may carry top-level documentation blocks alongside
    /// its suites. Parsed in document order; routed here regardless of where
    /// they appear thanks to quick-xml's `overlapped-lists` feature.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}

/// A `<suite>` groups related commands, events, classes, enumerations,
/// record-types, and value-types.
///
/// The DTD's `(class | command | enumeration | event | record-type |
/// value-type | documentation)+` content model permits child elements to
/// appear in any order, and Apple's own system sdefs interleave them
/// (e.g. `CocoaStandard.sdef` alternates commands and enumerations). We
/// rely on quick-xml's `overlapped-lists` cargo feature to deserialize
/// each child directly into its typed `Vec` field regardless of order.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Suite {
    /// Suite name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character suite code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

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

    /// All `<command>` children of this suite, in document order.
    #[serde(rename = "command", default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<Command>,

    /// All `<event>` children of this suite, in document order.
    #[serde(rename = "event", default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<Event>,

    /// All `<class>` children of this suite, in document order.
    #[serde(rename = "class", default, skip_serializing_if = "Vec::is_empty")]
    pub classes: Vec<Class>,

    /// All `<class-extension>` children of this suite, in document order.
    #[serde(
        rename = "class-extension",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub class_extensions: Vec<ClassExtension>,

    /// All `<enumeration>` children of this suite, in document order.
    #[serde(rename = "enumeration", default, skip_serializing_if = "Vec::is_empty")]
    pub enumerations: Vec<Enumeration>,

    /// All `<record-type>` children of this suite, in document order.
    #[serde(rename = "record-type", default, skip_serializing_if = "Vec::is_empty")]
    pub record_types: Vec<RecordType>,

    /// All `<value-type>` children of this suite, in document order.
    #[serde(rename = "value-type", default, skip_serializing_if = "Vec::is_empty")]
    pub value_types: Vec<ValueType>,

    /// `<documentation>` child blocks, in document order. Per DTD,
    /// documentation can interleave with other declarations inside a suite.
    #[serde(
        rename = "documentation",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub documentation: Vec<Documentation>,
}
