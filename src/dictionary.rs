//! Top-level structural AST nodes: [`Dictionary`] and [`Suite`].
//!
//! Modelled after the `<dictionary>` and `<suite>` elements defined in
//! `/System/Library/DTDs/sdef.dtd`. Verb-family types ([`crate::Command`],
//! [`crate::Event`]) live in [`crate::command`]; type/member declarations
//! ([`crate::Enumeration`], [`crate::RecordType`], [`crate::ValueType`],
//! [`crate::Property`]) live in [`crate::decl`].

use serde::Deserialize;

use crate::class::{Class, ClassExtension};
use crate::command::{Command, Event};
use crate::decl::{Enumeration, RecordType, ValueType};
use crate::metadata::{AccessGroup, Cocoa, Documentation};
use crate::yorn::yorn;

/// The root `<dictionary>` element of an sdef document.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "dictionary")]
pub struct Dictionary {
    /// Optional human-readable title attribute (`title="…"`).
    #[serde(rename = "@title", default)]
    pub title: Option<String>,

    /// All `<suite>` children. The DTD requires at least one suite.
    #[serde(rename = "suite", default)]
    pub suites: Vec<Suite>,
    // TODO(specialist): the DTD also permits zero-or-more <documentation>
    // siblings of <suite>. Model them when a consumer asks for it.
}

/// A `<suite>` groups related commands, events, classes, enumerations,
/// record-types, and value-types.
#[derive(Debug, Clone, Deserialize)]
pub struct Suite {
    /// Suite name (`name="…"`).
    #[serde(rename = "@name")]
    pub name: String,

    /// Four-character suite code (`code="…"`).
    #[serde(rename = "@code")]
    pub code: String,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// `hidden="yes"` flag, defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children (since OS X 10.8).
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// All `<command>` children of this suite.
    #[serde(rename = "command", default)]
    pub commands: Vec<Command>,

    /// All `<event>` children of this suite.
    #[serde(rename = "event", default)]
    pub events: Vec<Event>,

    /// All `<class>` children of this suite.
    #[serde(rename = "class", default)]
    pub classes: Vec<Class>,

    /// All `<class-extension>` children of this suite.
    #[serde(rename = "class-extension", default)]
    pub class_extensions: Vec<ClassExtension>,

    /// All `<enumeration>` children of this suite.
    #[serde(rename = "enumeration", default)]
    pub enumerations: Vec<Enumeration>,

    /// All `<record-type>` children of this suite.
    #[serde(rename = "record-type", default)]
    pub record_types: Vec<RecordType>,

    /// All `<value-type>` children of this suite.
    #[serde(rename = "value-type", default)]
    pub value_types: Vec<ValueType>,

    /// `<documentation>` child blocks. Per DTD, documentation can interleave
    /// with other declarations inside a suite; we collect them all here in
    /// document order.
    #[serde(rename = "documentation", default)]
    pub documentation: Vec<Documentation>,
}
