# Development

Contributor reference. End-user docs live in [`README.md`](README.md); the
conformance dashboard lives in [`docs/CONFORMANCE.md`](docs/CONFORMANCE.md).

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
  parses.rs                  — integration tests against tests/fixtures/
  dtd_drift.rs               — macOS-only DTD hash + xmllint drift detector
  attribute_conformance.rs   — DTD attributes vs. attribute manifest
  conformance_matrix.rs      — regenerates docs/CONFORMANCE.md; CI-enforced
  common/                    — shared manifest + DTD parser used above
  fixtures/
    synthetic.sdef             — comprehensive fixture, exercises every modelled element
    mini.sdef                  — minimum-valid dictionary
    extras.sdef                — edge cases (interleaved class-contents, etc.)
    sdef.dtd.sha256            — pinned hash of /System/Library/DTDs/sdef.dtd
    attribute_manifest.toml    — per-element DTD attribute set + closed-enum values
fuzz/
  Cargo.toml                 — independent cargo-fuzz crate
  fuzz_targets/
    parse_strict.rs            — strict-mode libFuzzer target
    parse_lenient.rs           — lenient-mode libFuzzer target
docs/
  CONFORMANCE.md             — generated conformance dashboard
```

## Building from source

The published crate is pure-Rust: depending on `sdef` from `Cargo.toml` pulls
in `quick-xml`, `serde`, and `thiserror` only.

The **test suite** additionally links against libxml2 — `tests/common/mod.rs`
uses the [`libxml`](https://crates.io/crates/libxml) crate to parse
`/System/Library/DTDs/sdef.dtd` for the attribute-conformance test, rather
than shipping its own DTD parser. Running `cargo test` therefore needs the
libxml2 C library plus `pkg-config`:

```sh
# Linux
sudo apt-get install -y libxml2-dev pkg-config

# macOS (Homebrew)
brew install libxml2 pkg-config
export PKG_CONFIG_PATH="$(brew --prefix libxml2)/lib/pkgconfig"
```

The canonical CI incantations live in
[`.github/workflows/ci.yml`](.github/workflows/ci.yml) and
[`.github/workflows/corpus-macos.yml`](.github/workflows/corpus-macos.yml).

## Implementation notes

- **Interleaved children.** The DTD permits `<suite>`, `<class>`, and
  `<class-extension>` children in any order, and Apple's own sdefs exercise
  this freely. We enable quick-xml's `overlapped-lists` cargo feature so each
  child element deserialises directly into its typed `Vec<T>` field regardless
  of document order.
