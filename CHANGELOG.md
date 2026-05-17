# Changelog

Per-version development history of this crate. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate
follows [SemVer](https://semver.org/spec/v2.0.0.html).

Versions 0.1.0 and 0.2.0 lived only as local milestones in the git
history. They were never released to crates.io; consumers should treat
0.3.0 as the first available version.

## [0.3.0] — 2026-05-17

First crates.io release.

### Added

- `#![forbid(unsafe_code)]` on the library crate.
- `Hash` derive on every public AST struct and enum.
- Infallible `FromStr` impls for `Access`, `AccessorStyle`, and
  `CocoaBooleanValue`, symmetric with the existing `Display` impls.
- `examples/dump.rs` showing how to walk a parsed dictionary, surfaced
  on docs.rs.
- MSRV CI job pinning to Rust 1.85 (build `--all-targets`,
  `cargo test --lib`) so accidental MSRV drift is caught on PRs.
- `[package.metadata.docs.rs]` with `all-features = true` and the
  conventional `docsrs` cfg.

### Fixed

- `<command>` and `<event>` emit `<direct-parameter>` before
  `<parameter>`, matching the DTD content model `(direct-parameter ,
  parameter*)`. The previous field order produced output that
  `xmllint --dtdvalid` rejected.

### Changed

- `Cargo.toml` `repository` field committed (previously stashed in a
  comment).

## [0.2.0] — 2026-05-16

Internal milestone, not published. Adds the QA arsenal and full
serialisation surface.

### Added

- `AccessorStyle` typed enum modelling the DTD's `accessor-type` entity
  (`index | name | id | range | relative | test`) with an `Other(String)`
  escape hatch.
- `Access` typed enum (`r | w | rw`) and `CocoaBooleanValue` typed enum
  (`YES | NO`), each with `Other(String)` escape hatches.
- `Dictionary::to_xml_string`, `to_writer`, `to_path` round-trip
  emission, producing a complete sdef document with the XML declaration
  and DOCTYPE prelude. AST identity (`parse(emit(parse(input))) ==
  parse(input)`) property-tested over every fixture.
- `fuzz/` crate with two `cargo-fuzz` libFuzzer targets
  (`parse_strict`, `parse_lenient`) seeded from
  `tests/fixtures/*.sdef`, plus a weekly `fuzz.yml` workflow that
  uploads crash reproducers as artifacts.
- `tests/attribute_conformance.rs` integration test that cross-checks
  every modelled attribute against the live `/System/Library/DTDs/sdef.dtd`
  via libxml2 on macOS, plus `tests/fixtures/attribute_manifest.toml` as
  the pinning ground truth.
- `tests/conformance_matrix.rs` generating `docs/CONFORMANCE.md` as a
  reviewer-facing coverage dashboard; CI fails on drift.
- Explicit pinning tests for XInclude / XXE / entity-expansion safety
  (later extended in 0.3.0).
- `DEVELOPMENT.md` with crate layout, build-time dependencies, and
  implementation notes split out of the README.

### Changed

- The `tests/common/mod.rs` helper exposes a `parse_dtd` function over
  libxml2 bindings, consumed by both `attribute_conformance` and
  `conformance_matrix`.

## [0.1.0] — 2026-05-14

Internal milestone, not published. Initial parser scaffolding through
full DTD coverage.

### Added

- Typed AST for every element in `/System/Library/DTDs/sdef.dtd`:
  `<dictionary>`, `<suite>`, `<command>`, `<event>`, `<parameter>`,
  `<direct-parameter>`, `<result>`, `<class>`, `<class-extension>`,
  `<contents>`, `<element>`, `<accessor>`, `<responds-to>`,
  `<property>`, `<enumeration>`, `<enumerator>`, `<record-type>`,
  `<value-type>`, `<type>`, `<synonym>`, `<documentation>`, `<html>`,
  `<xref>`, `<cocoa>`, `<access-group>` (25 element types).
- `Dictionary::from_str` / `from_reader` / `from_path` lenient parsing
  via quick-xml's serde layer with `overlapped-lists` for any-order
  child elements.
- `Dictionary::from_str_strict` (and reader/path siblings) running a
  fast event-level pre-pass that rejects unknown element names and
  `<xi:include>` directives.
- `Error::Io`, `Error::Xml`, `Error::UnknownElement`,
  `Error::XIncludeUnsupported` variants.
- `#[non_exhaustive]` on every public AST struct and enum.
- `CommandResult` named to avoid clashing with the prelude's `Result`.
- `Dictionary::command` / `class` / `suite` / `enumeration` lookup
  helpers.
- `tests/dtd_drift.rs` DTD-drift detection: SHA-256 of the system DTD
  pinned against `tests/fixtures/sdef.dtd.sha256`, plus per-fixture
  `xmllint --dtdvalid` validation. Self-skips on non-macOS.
- `tests/fixtures/{mini,synthetic,extras}.sdef` exercising the full
  modelled DTD surface.
- `tests/parses.rs`, `tests/roundtrip.rs`, `tests/security.rs`
  integration tests pinning happy-path parsing, AST-identity
  round-trip, and XInclude/XXE/entity-expansion safety.
- GitHub Actions CI: fmt, clippy, test, doc, audit (`ci.yml`).
- `corpus-macos.yml` workflow running `corpus_smoke` against
  `/System/Library/ScriptingDefinitions` and
  `/Applications/Xcode.app/Contents/Resources` on macOS-latest weekly,
  uploading `target/corpus_coverage.json` as an artifact.
- Dependabot configuration for `cargo` and `github-actions`.
- README with the full feature surface, strict-mode doctest, and
  per-OS-release schema-change table sourced from `sdef(5)`.

[0.3.0]: https://github.com/gronke/rust-sdef/releases/tag/v0.3.0
[0.2.0]: https://github.com/gronke/rust-sdef/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gronke/rust-sdef/tree/main
