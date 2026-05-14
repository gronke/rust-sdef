//! DTD drift detection.
//!
//! Runs whenever `/System/Library/DTDs/sdef.dtd` is present (typically only
//! on macOS); self-skips cleanly on ubuntu CI. Two concerns are checked
//! together so a single `cargo test` invocation surfaces both:
//!
//! 1. SHA-256 of the live system DTD must match the digest pinned in
//!    `tests/fixtures/sdef.dtd.sha256`. A mismatch means Apple has revised
//!    the schema since this crate was modelled; a human reviewer should
//!    diff `man 5 sdef`, update `src/strict.rs`'s `KNOWN_ELEMENTS` plus any
//!    missing AST coverage, then bump the pinned digest.
//!
//! 2. Every `tests/fixtures/*.sdef` must pass `xmllint --noout --dtdvalid`
//!    against the live system DTD. This catches DTD-violating constructs
//!    in our fixtures — independent of our parser, so it's a genuine
//!    second pair of eyes.
//!
//! Modes:
//! - default: runs both checks and fails at the end if either flagged a
//!   problem. Both stderr-print all the detail.
//! - `SDEF_DTD_STRICT=1`: panics immediately on hash mismatch, before
//!   running xmllint. Useful as a bail-out-early gate.

use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

const SYSTEM_DTD: &str = "/System/Library/DTDs/sdef.dtd";

#[test]
fn dtd_drift() {
    let dtd_path = Path::new(SYSTEM_DTD);
    if !dtd_path.exists() {
        eprintln!("(System DTD {SYSTEM_DTD} not present; skipping drift test)");
        return;
    }

    let pinned_hash = read_pinned_hash();
    let dtd_bytes = std::fs::read(dtd_path).expect("read system DTD");
    let actual_hash = hex_encode(Sha256::digest(&dtd_bytes).as_slice());

    let strict = std::env::var("SDEF_DTD_STRICT").as_deref() == Ok("1");
    let mut errors: Vec<String> = Vec::new();

    if actual_hash != pinned_hash {
        let msg = format!(
            "DTD drift detected:\n  pinned: {pinned_hash}\n  live:   {actual_hash}\n\
             Apple's {SYSTEM_DTD} has changed since this crate was modelled.\n\
             Review `man 5 sdef` for additions, update src/strict.rs's\n\
             KNOWN_ELEMENTS and the AST as needed, then bump\n\
             tests/fixtures/sdef.dtd.sha256."
        );
        eprintln!("{msg}");
        if strict {
            panic!("strict mode: bailing out early on DTD drift");
        }
        errors.push("hash mismatch".to_owned());
    }

    let fixtures = collect_fixtures();
    if fixtures.is_empty() {
        panic!("no .sdef fixtures found under tests/fixtures/ — check working directory");
    }

    let mut cmd = Command::new("xmllint");
    cmd.arg("--noout").arg("--dtdvalid").arg(SYSTEM_DTD);
    for fx in &fixtures {
        cmd.arg(fx);
    }
    match cmd.output() {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "xmllint --dtdvalid reported failures over {} fixture(s):\n{stderr}",
                    fixtures.len()
                );
                errors.push("xmllint reported invalid fixtures".to_owned());
            } else {
                eprintln!(
                    "xmllint --dtdvalid: {} fixture(s) validate cleanly against the live DTD",
                    fixtures.len()
                );
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "(xmllint not installed; skipping fixture DTD-validation. \
                 macOS ships xmllint at /usr/bin/xmllint; \
                 on Ubuntu install libxml2-utils.)"
            );
        }
        Err(e) => panic!("failed to invoke xmllint: {e}"),
    }

    if !errors.is_empty() {
        panic!(
            "dtd_drift detected {} problem(s) — see stderr above for details: {:?}",
            errors.len(),
            errors
        );
    }
}

fn read_pinned_hash() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sdef.dtd.sha256");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read pinned hash file {}: {e}", path.display()));
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .unwrap_or_else(|| panic!("pinned hash file must contain a non-comment line"))
        .to_owned()
}

fn collect_fixtures() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("read tests/fixtures: {e}"));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "sdef") {
            out.push(path);
        }
    }
    out.sort();
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
