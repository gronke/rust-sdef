//! Cross-cutting metadata elements: `<cocoa>`, `<access-group>`.
//!
//! These appear as optional children on many sdef elements (commands,
//! suites, classes, properties, ...) and carry implementation details or
//! sandbox entitlements rather than scripting terminology.

use serde::Deserialize;

use crate::yorn::yorn;

/// Implementation hint for Cocoa Scripting (the `<cocoa>` element).
///
/// Cocoa applications use these attributes to wire AppleScript terms to
/// Objective-C classes, methods, and KVC keys. All fields are optional —
/// when omitted, Cocoa Scripting derives defaults from the surrounding term
/// (camel-cased names, pluralised element keys, etc.). See `man sdef` for
/// the full rules.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Cocoa {
    /// `name="…"` — legacy scriptSuite-compatibility attribute carried on
    /// suites, verbs, enumerations, and enumerators.
    #[serde(rename = "@name", default)]
    pub name: Option<String>,

    /// `class="…"` — Objective-C class implementing this scripting term.
    /// Common on `<command>` (the `NSScriptCommand` subclass) and `<class>`
    /// (the modelled NSObject subclass).
    #[serde(rename = "@class", default)]
    pub class: Option<String>,

    /// `key="…"` — Key-Value-Coding key for property/element access, or the
    /// NSDictionary key for command parameters. Replaced the older `method`
    /// attribute on properties/elements in OS X 10.4.
    #[serde(rename = "@key", default)]
    pub key: Option<String>,

    /// `method="…"` — Objective-C selector, used on `<responds-to>` to map
    /// a scripting verb to a method implementation.
    #[serde(rename = "@method", default)]
    pub method: Option<String>,

    /// `insert-at-beginning="yes|no"` — only meaningful on `<element>`.
    /// Defaults to `false` when absent.
    #[serde(rename = "@insert-at-beginning", default, deserialize_with = "yorn")]
    pub insert_at_beginning: bool,

    /// `boolean-value="YES|NO"` — literal value (note the capital-case
    /// `YES`/`NO`, distinct from the `yorn` lowercase entity used by other
    /// attributes). Kept as a raw `String` so callers can interpret without
    /// us swallowing unknown variants.
    #[serde(rename = "@boolean-value", default)]
    pub boolean_value: Option<String>,

    /// `string-value="…"` — arbitrary literal string value.
    #[serde(rename = "@string-value", default)]
    pub string_value: Option<String>,

    /// `integer-value="…"` — literal integer value as written in the source
    /// XML. Kept as `String` to preserve formatting and avoid lossy
    /// conversions; callers can parse into `i64` as needed.
    #[serde(rename = "@integer-value", default)]
    pub integer_value: Option<String>,
}

/// Sandbox entitlement: the `<access-group>` element (added in OS X 10.8).
///
/// Restricts which sandboxed apps may invoke this command (or access this
/// property/element) by matching the requesting app's identifier against
/// `identifier`. The wildcard `"*"` matches any caller.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AccessGroup {
    /// `identifier="…"` — reverse-DNS-style sandbox identifier, or `"*"`
    /// for an unrestricted wildcard.
    #[serde(rename = "@identifier")]
    pub identifier: String,

    /// `access="r|w|rw"` — meaningful only when the access-group is attached
    /// to a property or element; restricts whether the matching apps get
    /// read, write, or read-write access. `None` means the attribute was
    /// omitted (typical for `<access-group>` on `<command>` and `<suite>`).
    #[serde(rename = "@access", default)]
    pub access: Option<String>,
}
