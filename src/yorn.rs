//! Helper for the DTD's `(yes | no)` entity.
//!
//! Several sdef attributes (`optional`, `hidden`, `inherited`, …) are typed as
//! the `yorn` entity in `sdef.dtd`. This module supplies a serde deserializer
//! that turns the literal strings `"yes"`/`"no"` into `bool`, treating
//! "attribute absent" as `false`, plus a matching serializer that emits the
//! same literals on the way back out.

use serde::{Deserialize, Deserializer, Serializer};

/// `serde` deserializer for DTD `(yes | no)` attributes.
///
/// Intended for use with `#[serde(deserialize_with = "yorn::de")]` on a
/// `bool` field. Absent attributes deserialize to `false` (combine with
/// `#[serde(default)]`).
pub(crate) fn de<'de, D>(deserializer: D) -> Result<bool, D::Error>
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

/// Like [`de`], but distinguishes "attribute absent" (`None`) from the
/// explicit `yes`/`no` values. Use this when the DTD's default for an
/// absent attribute is not `false` (e.g. `property.in-properties` defaults
/// to `yes` per the sdef man page).
pub(crate) fn de_opt<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
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

/// `serde` serializer mirroring [`de`]. Emits the literal string `"yes"`
/// for `true` and `"no"` for `false`. Pair with
/// `#[serde(skip_serializing_if = "yorn::is_false")]` to omit the
/// attribute entirely when the value is `false` (the DTD's implicit
/// default).
pub(crate) fn ser<S: Serializer>(b: &bool, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(if *b { "yes" } else { "no" })
}

/// Optional-variant serializer mirroring [`de_opt`]. Emits `"yes"` /
/// `"no"` for `Some(true)` / `Some(false)`, and a `None` (omitted
/// attribute) for `None`.
pub(crate) fn ser_opt<S: Serializer>(b: &Option<bool>, s: S) -> Result<S::Ok, S::Error> {
    match b {
        Some(true) => s.serialize_str("yes"),
        Some(false) => s.serialize_str("no"),
        None => s.serialize_none(),
    }
}

/// Predicate for `#[serde(skip_serializing_if = "yorn::is_false")]`. Lets
/// us drop `bool` attributes whose effective DTD default is `false`,
/// keeping emitted XML compact.
pub(crate) fn is_false(b: &bool) -> bool {
    !*b
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Probe {
        #[serde(
            rename = "@flag",
            default,
            deserialize_with = "de",
            serialize_with = "ser",
            skip_serializing_if = "is_false"
        )]
        flag: bool,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ProbeOpt {
        #[serde(
            rename = "@flag",
            default,
            deserialize_with = "de_opt",
            serialize_with = "ser_opt",
            skip_serializing_if = "Option::is_none"
        )]
        flag: Option<bool>,
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

    #[test]
    fn serializes_true_as_yes() {
        let p = Probe { flag: true };
        let xml = quick_xml::se::to_string(&p).expect("serialize");
        assert!(xml.contains(r#"flag="yes""#), "got {xml}");
    }

    #[test]
    fn serializes_false_omits_attribute() {
        let p = Probe { flag: false };
        let xml = quick_xml::se::to_string(&p).expect("serialize");
        assert!(
            !xml.contains("flag="),
            "expected attribute omitted, got {xml}"
        );
    }

    #[test]
    fn opt_serializes_distinct_states() {
        let yes = quick_xml::se::to_string(&ProbeOpt { flag: Some(true) }).expect("ser yes");
        let no = quick_xml::se::to_string(&ProbeOpt { flag: Some(false) }).expect("ser no");
        let none = quick_xml::se::to_string(&ProbeOpt { flag: None }).expect("ser none");
        assert!(yes.contains(r#"flag="yes""#), "got {yes}");
        assert!(no.contains(r#"flag="no""#), "got {no}");
        assert!(!none.contains("flag="), "got {none}");
    }
}
