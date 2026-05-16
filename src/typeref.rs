//! The `<type>` child element used inside parameters, results, properties,
//! and similar sites to express type references.
//!
//! Distinct from `<value-type>` and `<record-type>` (which *declare* simple
//! types in a suite). A `<type>` element here is a *reference* to a primitive
//! or a previously declared user type, optionally marked as a list and
//! optionally containing nested `<type>` children for union expressions.

use serde::Deserialize;

use crate::yorn::yorn;

/// A typed reference: the `<type>` child element.
///
/// Used wherever the DTD permits `(type | documentation)*` — including
/// `<parameter>`, `<direct-parameter>`, `<result>`, `<property>`,
/// `<contents>`, and inside other `<type>` elements for union expressions.
///
/// The `type` attribute names either one of the documented primitives
/// (`any`, `text`, `integer`, `real`, `number`, `boolean`, `specifier`,
/// `location specifier`, `record`, `date`, `file`, `point`, `rectangle`,
/// `type`, `missing value`) or a user-declared class/enumeration/record-type/
/// value-type by name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TypeRef {
    /// `type="…"` — primitive name or user-declared type reference.
    #[serde(rename = "@type")]
    pub ty: String,

    /// `list="yes|no"` — collection-of-`type` marker. Defaults to `false`.
    /// Nested-list expressions ("list of list of …") are not supported by
    /// Cocoa Scripting; the DTD permits the markup but it has no effect.
    #[serde(rename = "@list", default, deserialize_with = "yorn")]
    pub list: bool,

    /// `hidden="yes|no"` — defaults to `false`.
    #[serde(rename = "@hidden", default, deserialize_with = "yorn")]
    pub hidden: bool,

    /// Nested `<type>` children. Per the DTD, `<type>` elements can
    /// recursively contain other `<type>` elements; in practice this is
    /// rare. Most union expressions appear as multiple sibling `<type>`
    /// elements within a single parent (parameter/result/property) rather
    /// than as nested children.
    #[serde(rename = "type", default)]
    pub types: Vec<TypeRef>,
}
