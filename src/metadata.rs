//! Cross-cutting metadata elements: `<cocoa>`, `<access-group>`, `<synonym>`,
//! `<documentation>` (and its `<html>` children), `<xref>`.
//!
//! Also home to the cross-cutting closed-enum types `Access` (for
//! `@access` and `@requires-access`) and `CocoaBooleanValue` (for
//! `<cocoa boolean-value="…">`). Each typed enum mirrors the DTD's
//! allowed-value set, with an `Other(String)` escape hatch for forward
//! compatibility against future Apple additions.
//!
//! These appear as optional children on many sdef elements (commands,
//! suites, classes, properties, ...) and carry implementation details,
//! sandbox entitlements, or human-readable annotations rather than
//! scripting terminology.

use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::yorn;

/// The closed set of `<… access="…">` and `<… requires-access="…">` values
/// declared by the sdef DTD: `(r | w | rw)`.
///
/// Used by `<access-group>`, `<contents>`, `<element>`, `<property>`,
/// `<parameter>`, and `<direct-parameter>`. Unknown values deserialize to
/// [`Access::Other`] for lenient mode; the variant list is kept in
/// lock-step with the DTD via the `attribute_conformance` integration
/// test.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum Access {
    /// `access="r"` — read-only.
    Read,
    /// `access="w"` — write-only.
    Write,
    /// `access="rw"` — read/write.
    ReadWrite,
    /// Unrecognised value, preserved verbatim for lenient parsing.
    Other(String),
}

impl Access {
    /// Returns the canonical DTD string for this variant, or the wrapped
    /// value for [`Access::Other`]. Mirrors the textual form the parser
    /// accepts.
    pub fn as_str(&self) -> &str {
        match self {
            Access::Read => "r",
            Access::Write => "w",
            Access::ReadWrite => "rw",
            Access::Other(s) => s,
        }
    }
}

impl fmt::Display for Access {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Access {
    type Err = Infallible;

    /// Infallible: unknown values map to [`Access::Other`], matching the
    /// lenient-deserialize behaviour. The strict-mode parsing path
    /// ([`crate::Dictionary::from_str_strict`]) is the place to reject
    /// out-of-range values at document level.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "r" => Access::Read,
            "w" => Access::Write,
            "rw" => Access::ReadWrite,
            _ => Access::Other(s.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for Access {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "r" => Access::Read,
            "w" => Access::Write,
            "rw" => Access::ReadWrite,
            _ => Access::Other(raw),
        })
    }
}

impl Serialize for Access {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// The closed set of `<cocoa boolean-value="…">` values: `(YES | NO)`.
///
/// Note the capital-case spelling — distinct from the lowercase `yorn`
/// entity used by `@hidden`, `@optional`, and friends. Lenient parsing
/// preserves any unrecognised value via [`CocoaBooleanValue::Other`];
/// strict mode rejects unknown values.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
#[non_exhaustive]
pub enum CocoaBooleanValue {
    /// `boolean-value="YES"`.
    Yes,
    /// `boolean-value="NO"`.
    No,
    /// Unrecognised value, preserved verbatim for lenient parsing.
    Other(String),
}

impl CocoaBooleanValue {
    /// Returns the canonical DTD string for this variant, or the wrapped
    /// value for [`CocoaBooleanValue::Other`].
    pub fn as_str(&self) -> &str {
        match self {
            CocoaBooleanValue::Yes => "YES",
            CocoaBooleanValue::No => "NO",
            CocoaBooleanValue::Other(s) => s,
        }
    }
}

impl fmt::Display for CocoaBooleanValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CocoaBooleanValue {
    type Err = Infallible;

    /// Infallible: unknown values map to [`CocoaBooleanValue::Other`].
    /// Strict-mode parsing rejects values outside `{YES, NO}` at the
    /// document level via [`crate::Dictionary::from_str_strict`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "YES" => CocoaBooleanValue::Yes,
            "NO" => CocoaBooleanValue::No,
            _ => CocoaBooleanValue::Other(s.to_owned()),
        })
    }
}

impl<'de> Deserialize<'de> for CocoaBooleanValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "YES" => CocoaBooleanValue::Yes,
            "NO" => CocoaBooleanValue::No,
            _ => CocoaBooleanValue::Other(raw),
        })
    }
}

impl Serialize for CocoaBooleanValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Implementation hint for Cocoa Scripting (the `<cocoa>` element).
///
/// Cocoa applications use these attributes to wire AppleScript terms to
/// Objective-C classes, methods, and KVC keys. All fields are optional —
/// when omitted, Cocoa Scripting derives defaults from the surrounding term
/// (camel-cased names, pluralised element keys, etc.). See `man sdef` for
/// the full rules.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Cocoa {
    /// `name="…"` — legacy scriptSuite-compatibility attribute carried on
    /// suites, verbs, enumerations, and enumerators.
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// `class="…"` — Objective-C class implementing this scripting term.
    /// Common on `<command>` (the `NSScriptCommand` subclass) and `<class>`
    /// (the modelled NSObject subclass).
    #[serde(rename = "@class", default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,

    /// `key="…"` — Key-Value-Coding key for property/element access, or the
    /// NSDictionary key for command parameters. Replaced the older `method`
    /// attribute on properties/elements in OS X 10.4.
    #[serde(rename = "@key", default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// `method="…"` — Objective-C selector, used on `<responds-to>` to map
    /// a scripting verb to a method implementation.
    #[serde(rename = "@method", default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// `insert-at-beginning="yes|no"` — only meaningful on `<element>`.
    /// Defaults to `false` when absent.
    #[serde(
        rename = "@insert-at-beginning",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub insert_at_beginning: bool,

    /// `boolean-value="YES|NO"` — literal value (note the capital-case
    /// `YES`/`NO`, distinct from the `yorn` lowercase entity used by other
    /// attributes). Surfaced as a typed [`CocoaBooleanValue`] with an
    /// `Other(String)` escape hatch for forward compatibility.
    #[serde(
        rename = "@boolean-value",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub boolean_value: Option<CocoaBooleanValue>,

    /// `string-value="…"` — arbitrary literal string value.
    #[serde(
        rename = "@string-value",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub string_value: Option<String>,

    /// `integer-value="…"` — literal integer value as written in the source
    /// XML. Kept as `String` to preserve formatting and avoid lossy
    /// conversions; callers can parse into `i64` as needed.
    #[serde(
        rename = "@integer-value",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub integer_value: Option<String>,
}

/// Sandbox entitlement: the `<access-group>` element (added in OS X 10.8).
///
/// Restricts which sandboxed apps may invoke this command (or access this
/// property/element) by matching the requesting app's identifier against
/// `identifier`. The wildcard `"*"` matches any caller.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct AccessGroup {
    /// `identifier="…"` — reverse-DNS-style sandbox identifier, or `"*"`
    /// for an unrestricted wildcard.
    #[serde(rename = "@identifier")]
    pub identifier: String,

    /// `access="r|w|rw"` — meaningful only when the access-group is attached
    /// to a property or element; restricts whether the matching apps get
    /// read, write, or read-write access. `None` means the attribute was
    /// omitted (typical for `<access-group>` on `<command>` and `<suite>`).
    #[serde(rename = "@access", default, skip_serializing_if = "Option::is_none")]
    pub access: Option<Access>,
}

/// An alternate scripting term or code: the `<synonym>` element.
///
/// Most terminology elements may declare synonyms so AppleScript code can
/// refer to them by a second name (or, less commonly, a second OSType code)
/// without losing the canonical mapping. At least one of `name`/`code` is
/// required by the DTD; we treat both as optional here and let downstream
/// validation enforce that.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Synonym {
    /// `name="…"` — alternate scripting term.
    #[serde(rename = "@name", default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// `code="…"` — alternate four-character OSType code.
    #[serde(rename = "@code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// `hidden="yes|no"` — synonyms hidden from dictionary viewers; defaults
    /// to `false`. Code-only synonyms are implicitly hidden per the DTD.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,

    /// `plural="…"` — alternate plural form (class-style synonyms only).
    #[serde(rename = "@plural", default, skip_serializing_if = "Option::is_none")]
    pub plural: Option<String>,

    /// Optional `<cocoa>` implementation hint child (required by the DTD when
    /// the synonym carries a code-only or name-and-code form).
    #[serde(rename = "cocoa", default, skip_serializing_if = "Option::is_none")]
    pub cocoa: Option<Cocoa>,
}

/// Human-readable documentation block: the `<documentation>` element.
///
/// May appear on the dictionary, suites, any terminology element, and inside
/// `<parameter>`/`<direct-parameter>`/`<result>` (since OS X 10.10). Each
/// block holds one or more `<html>` snippets — escaped HTML text since
/// OS X 10.5, raw text in earlier releases.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Documentation {
    /// `<html>` text snippets. Each entry is the inner text of one `<html>`
    /// element, in document order. quick-xml decodes XML entities here, so
    /// the strings are ready for display (or further HTML-escape decoding by
    /// the caller for pre-10.5 dictionaries).
    #[serde(rename = "html", default, skip_serializing_if = "Vec::is_empty")]
    pub html: Vec<String>,
}

/// Cross-reference: the `<xref>` element (added in OS X 10.5).
///
/// Documentation-only pointer to another scriptability element by name or
/// id. No semantic effect on scripting behaviour; consumed by dictionary
/// browsers.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Xref {
    /// `target="…"` — name or id of the referenced element.
    #[serde(rename = "@target")]
    pub target: String,

    /// `hidden="yes|no"` — defaults to `false`.
    #[serde(
        rename = "@hidden",
        default,
        deserialize_with = "yorn::de",
        serialize_with = "yorn::ser",
        skip_serializing_if = "yorn::is_false"
    )]
    pub hidden: bool,
}
