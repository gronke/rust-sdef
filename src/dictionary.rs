//! Top-level AST nodes: [`Dictionary`], [`Suite`], [`Command`].
//!
//! Modelled after the `<dictionary>`, `<suite>`, and `<command>` elements
//! defined in `/System/Library/DTDs/sdef.dtd`. Only the attributes and child
//! relationships this crate currently exposes are listed; extend as consumers
//! need more of the DTD.

use serde::Deserialize;

use crate::metadata::{AccessGroup, Cocoa};
use crate::parameter::{DirectParameter, Parameter, Result_};

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

/// A `<suite>` groups related commands, classes, and enumerations.
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

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children (since OS X 10.8).
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// All `<command>` children of this suite.
    #[serde(rename = "command", default)]
    pub commands: Vec<Command>,
    // TODO(specialist): also model <class>, <enumeration>, <record-type>,
    // <value-type>, <class-extension>, <event>.
}

/// A `<command>` — a verb the application supports via Apple Events.
#[derive(Debug, Clone, Deserialize)]
pub struct Command {
    /// Human-readable command name (`name="…"`), e.g. `"export transactions"`.
    #[serde(rename = "@name")]
    pub name: String,

    /// Eight-character Apple Event code (`code="…"`), e.g. `"MONYexpt"`.
    #[serde(rename = "@code")]
    pub code: String,

    /// Optional human description (`description="…"`).
    #[serde(rename = "@description", default)]
    pub description: Option<String>,

    /// Optional `<cocoa>` implementation hint child.
    #[serde(rename = "cocoa", default)]
    pub cocoa: Option<Cocoa>,

    /// Zero or more `<access-group>` entitlement children (since OS X 10.8).
    #[serde(rename = "access-group", default)]
    pub access_groups: Vec<AccessGroup>,

    /// `<parameter>` children, in document order.
    #[serde(rename = "parameter", default)]
    pub parameters: Vec<Parameter>,

    /// Optional `<direct-parameter>` (the un-named first argument).
    #[serde(rename = "direct-parameter", default)]
    pub direct_parameter: Option<DirectParameter>,

    /// Optional `<result>` element describing the command's return value.
    #[serde(rename = "result", default)]
    pub result: Option<Result_>,
    // TODO(specialist): <synonym>, <documentation>.
}
