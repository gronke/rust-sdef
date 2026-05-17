# Changelog

All notable changes to this crate are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate
follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.3.0] — 2026-05-17

Initial release on crates.io.

### Added

- Typed AST covering every element in `/System/Library/DTDs/sdef.dtd` as
  of macOS 26.x: `<dictionary>`, `<suite>`, `<command>`, `<event>`,
  `<parameter>`, `<direct-parameter>`, `<result>`, `<class>`,
  `<class-extension>`, `<contents>`, `<element>`, `<accessor>`,
  `<responds-to>`, `<property>`, `<enumeration>`, `<enumerator>`,
  `<record-type>`, `<value-type>`, `<type>`, `<synonym>`,
  `<documentation>`, `<html>`, `<xref>`, `<cocoa>`, `<access-group>`.
- Lenient parsing via `Dictionary::from_str` / `from_reader` /
  `from_path`, tolerating unknown XML elements for forward
  compatibility with future Apple additions.
- Strict parsing via `Dictionary::from_str_strict` (and reader/path
  siblings), rejecting unmodelled element names, `<xi:include>`
  directives, and out-of-range values for the closed-enumeration
  attributes.
- Round-trip emission via `Dictionary::to_xml_string` / `to_writer` /
  `to_path`, preserving AST identity.
- Typed enums for the DTD's closed-enumeration attributes:
  `AccessorStyle` (`accessor.style`), `Access` (`@access` and
  `@requires-access`), and `CocoaBooleanValue` (`cocoa.boolean-value`),
  each with an `Other(String)` escape hatch for lenient parsing.
- Dictionary lookup helpers: `Dictionary::command`, `class`, `suite`,
  `enumeration`.
- `Hash`, `FromStr`, `Display`, `Serialize`, `Deserialize`, and
  `#[non_exhaustive]` on the public AST surface.
- `examples/dump.rs` showing how to walk a parsed dictionary.
- `#![forbid(unsafe_code)]` on the library crate.
- MSRV: Rust 1.85.

[0.3.0]: https://github.com/gronke/rust-sdef/releases/tag/v0.3.0
