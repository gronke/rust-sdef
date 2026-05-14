//! Helper for the DTD's `(yes | no)` entity.
//!
//! Several sdef attributes (`optional`, `hidden`, `inherited`, …) are typed as
//! the `yorn` entity in `sdef.dtd`. This module supplies a serde deserializer
//! that turns the literal strings `"yes"`/`"no"` into `bool`, treating
//! "attribute absent" as `false`.

use serde::{Deserialize, Deserializer};

/// `serde` deserializer for DTD `(yes | no)` attributes.
///
/// Intended for use with `#[serde(deserialize_with = "yorn")]` on a `bool`
/// field. Absent attributes deserialize to `false` (combine with
/// `#[serde(default)]`).
pub(crate) fn yorn<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw.as_deref() {
        Some("yes") => Ok(true),
        Some("no") | None => Ok(false),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected 'yes' or 'no', got {other:?}"
        ))),
    }
}

/// Like [`yorn`], but distinguishes "attribute absent" (`None`) from the
/// explicit `yes`/`no` values. Use this when the DTD's default for an
/// absent attribute is not `false` (e.g. `property.in-properties` defaults
/// to `yes` per the sdef man page).
pub(crate) fn yorn_opt<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw.as_deref() {
        Some("yes") => Ok(Some(true)),
        Some("no") => Ok(Some(false)),
        None => Ok(None),
        Some(other) => Err(serde::de::Error::custom(format!(
            "expected 'yes' or 'no', got {other:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Probe {
        #[serde(rename = "@flag", default, deserialize_with = "yorn")]
        flag: bool,
    }

    fn parse(xml: &str) -> Probe {
        quick_xml::de::from_str(xml).expect("test input must parse")
    }

    #[test]
    fn parses_yes() {
        assert!(parse(r#"<probe flag="yes"/>"#).flag);
    }

    #[test]
    fn parses_no() {
        assert!(!parse(r#"<probe flag="no"/>"#).flag);
    }

    #[test]
    fn missing_defaults_to_false() {
        assert!(!parse(r#"<probe/>"#).flag);
    }

    #[test]
    fn rejects_garbage() {
        let err =
            quick_xml::de::from_str::<Probe>(r#"<probe flag="maybe"/>"#).expect_err("must fail");
        assert!(err.to_string().contains("expected 'yes' or 'no'"));
    }
}
