# sdef

Typed parser for Apple [scripting definition][sdef-man] (`.sdef`) XML files in Rust.

`.sdef` files are the modern, machine-readable terminology format that
AppleScript-able macOS applications ship to describe the commands, classes,
and enumerations they expose via Apple Events. Every scriptable app on macOS
ships one inside its bundle at `Contents/Resources/<Name>.sdef`, and Apple's
own `/System/Library/ScriptingDefinitions/` contains framework-level examples
such as `CocoaStandard.sdef`.

## Status

Full DTD coverage of the macOS 26 `sdef(5)` schema. Validated end-to-end
against four real-world sdefs from `/Applications/` and
`/System/Library/ScriptingDefinitions/` (CocoaStandard, MoneyMoney, Xcode,
Google Chrome). The crate is **not yet published to crates.io** — it lives
on GitHub and is pinned by `rust-moneymoney` as a git dependency while
both projects bed in together. See [Roadmap](#roadmap).

## Features

- **Full DTD coverage** of every element defined in
  `/System/Library/DTDs/sdef.dtd`: dictionary, suite, command, event,
  parameter, direct-parameter, result, class, class-extension, contents,
  element, accessor, responds-to, property, enumeration, enumerator,
  record-type, value-type, type, synonym, documentation, html, xref,
  cocoa, access-group.
- **Two parsing modes**:
  - `Dictionary::from_str` (and `from_path` / `from_reader`) tolerates
    unknown XML elements, which matches quick-xml's default behaviour and
    keeps the parser forward-compatible against future Apple additions.
  - `Dictionary::from_str_strict` (and friends) rejects any element name
    outside the modelled DTD subset, plus any `<xi:include>` directive.
    Useful for CI gates, tests, and any case where you want loud failures
    on DTD drift.
- **DTD-drift detection** baked into the test suite. The
  `tests/dtd_drift.rs` integration test checks two things together on
  every `cargo test`:
  1. SHA-256 of `/System/Library/DTDs/sdef.dtd` against a pinned digest
     in `tests/fixtures/sdef.dtd.sha256`. A mismatch fails the test and
     points the human reviewer at `man 5 sdef` for the change history.
  2. Every `tests/fixtures/*.sdef` validates against the live system DTD
     via `xmllint --noout --dtdvalid` — an independent validator catching
     anything our Rust-side parser would miss.

  Self-skips on non-macOS where the system DTD is absent. Set
  `SDEF_DTD_STRICT=1` to bail out early on hash mismatch instead of
  proceeding to xmllint.
- **`#[non_exhaustive]`** on every public AST struct so additive
  evolutions of the schema don't ratchet semver breaks for downstream
  consumers.

## Quick start

```rust
use sdef::Dictionary;

let xml = std::fs::read_to_string(
    "/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef",
)?;

// Lenient — silently drops unknown XML elements (forward-compat).
let dict: Dictionary = xml.parse()?;

for suite in &dict.suites {
    for command in &suite.commands {
        println!("{} ({})", command.name, command.code);
        if let Some(cocoa) = &command.cocoa {
            if let Some(class) = &cocoa.class {
                println!("  cocoa class: {class}");
            }
        }
        for group in &command.access_groups {
            println!("  entitlement: {}", group.identifier);
        }
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Strict mode for the same input:

```rust
use sdef::{Dictionary, Error};

let xml = std::fs::read_to_string("./MoneyMoney.sdef")?;
match Dictionary::from_str_strict(&xml) {
    Ok(dict) => println!("strict parse OK, {} suites", dict.suites.len()),
    Err(Error::UnknownElement { name }) => {
        eprintln!("DTD drift: unmodelled element <{name}>");
    }
    Err(Error::XIncludeUnsupported) => {
        eprintln!("resolve XIncludes before parsing");
    }
    Err(e) => return Err(e.into()),
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Dictionary` also implements [`FromStr`][fromstr], so the `.parse()` form is
the preferred entry point for the lenient path.

[fromstr]: https://doc.rust-lang.org/std/str/trait.FromStr.html

## Crate layout

```
src/
  lib.rs       — entry points, re-exports, crate-level docs
  dictionary.rs — Dictionary, Suite
  command.rs   — Command, Event, Parameter, DirectParameter, CommandResult
  class.rs     — Class, ClassExtension, Contents, Element, Accessor, RespondsTo
  decl.rs      — Enumeration, Enumerator, RecordType, ValueType, Property
  metadata.rs  — Cocoa, AccessGroup, Synonym, Documentation, Xref
  typeref.rs   — TypeRef (the <type> child element)
  strict.rs    — Strict-mode pre-validation pass
  error.rs     — Error enum
  yorn.rs      — (yes | no) entity → bool serde helper
tests/
  parses.rs        — integration tests against tests/fixtures/
  dtd_drift.rs     — macOS-only DTD hash + xmllint drift detector
  fixtures/
    synthetic.sdef    — comprehensive fixture, exercises every modelled element
    mini.sdef         — minimum-valid dictionary
    extras.sdef       — edge cases (interleaved class-contents, etc.)
    sdef.dtd.sha256   — pinned hash of /System/Library/DTDs/sdef.dtd
```

## DTD version coverage

The AST models the DTD shipped on **macOS 26.x** (`/System/Library/DTDs/sdef.dtd`,
sha256 in `tests/fixtures/sdef.dtd.sha256`). Per the `sdef(5)` man page's
change history:

| OS X / macOS release | Schema changes covered |
| -------------------- | ----------------------- |
| 10.4 (Tiger)         | sdef format baseline; collector elements removed; primitives renamed (`string`→`text`, `object`→`specifier`); complex types via `<type>` child; `<cocoa>` `method` → `key` for property/element; `not-in-properties` → `in-properties` |
| 10.5 (Leopard)       | `<xref>` element; `<html>` requires escaping (entities or CDATA); `<responds-to>` `name` → `command` (legacy `name` still accepted) |
| 10.8 (Mountain Lion) | `<access-group>` element; `<cocoa>` optional for class/parameter/responds-to/value-type |
| 10.10 (Yosemite)     | `<class-extension>` `title` attribute; `<documentation>` may appear inside `<parameter>` / `<direct-parameter>` / `<result>` |

Individual fields and struct-level rustdoc carry `(since OS X 10.X)` markers
where relevant so consumers can reason about what's available in older
sdefs they encounter.

## Limitations

- **`<xi:include>` is not resolved.** Strict mode rejects it with
  `Error::XIncludeUnsupported`; lenient mode silently treats it as a no-op
  (matching quick-xml). If you need XInclude resolution, pre-process the
  XML with `xmllint --xinclude` before handing it to this crate.
- **No DTD validation at the Rust layer.** quick-xml does not perform DTD
  validation, and strict mode only checks element names against the
  modelled set — not attribute names or value enumerations. For full
  validation, rely on the `xmllint --dtdvalid` step that the drift test
  wires up automatically on macOS.
- **No serialisation.** The crate only deserialises. Round-trip
  emission is on the roadmap but blocks on quick-xml-serde quirks around
  mixed content in `<html>` blocks.

## Implementation notes

- **Interleaved children.** The DTD permits `<suite>`, `<class>`, and
  `<class-extension>` children in any order, and Apple's own sdefs
  exercise this freely. We enable quick-xml's `overlapped-lists` cargo
  feature so each child element deserialises directly into its typed
  `Vec<T>` field regardless of document order.

## Roadmap

- [ ] Round-trip `Serialize` derive with property-based equivalence tests.
- [ ] CI matrix: Linux only is sufficient for the parser; macOS for the
      drift test. Already configured per the modelled CI workflow.
- [ ] First publish to crates.io. Pre-flight: pin a sensible MSRV, set
      the `repository` field in `Cargo.toml`, freeze the API surface.

## References

- **`sdef(5)` man page**: <https://keith.github.io/xcode-man-pages/sdef.5.html>
- **Authoritative DTD** ships with macOS at
  [`/System/Library/DTDs/sdef.dtd`](file:///System/Library/DTDs/sdef.dtd).
- **Apple's archived guide**: [_Preparing a Scripting Definition File_][apple-prepsdef].
- **Apple-provided samples**: `/System/Library/ScriptingDefinitions/`
  (e.g. `CocoaStandard.sdef`).

## License

MIT. See [LICENSE](LICENSE).

[sdef-man]: https://keith.github.io/xcode-man-pages/sdef.5.html
[apple-prepsdef]: https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/ScriptableCocoaApplications/SApps_creating_sdef/SAppsCreateSdef.html
