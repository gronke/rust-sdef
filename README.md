# sdef

Typed parser for Apple [scripting definition][sdef-man] (`.sdef`) XML files in Rust.

`.sdef` files are the modern, machine-readable terminology format that AppleScript-able
macOS applications ship to describe the commands, classes, and enumerations they
expose via Apple Events. Every scriptable app on macOS ships one inside its bundle at
`Contents/Resources/<Name>.sdef`, and Apple's own `/System/Library/ScriptingDefinitions/`
contains framework-level examples.

## Status

**Early scaffolding.** The crate compiles, the AST is sketched, and a minimal entry
point exists, but coverage of the DTD is incomplete and there are no real-world
integration tests yet. See [Roadmap](#roadmap).

## Motivation

There is no existing Rust sdef parser on crates.io or GitHub — the closest prior art
is BushelScript's Swift [SDEFinitely][sdefinitely], descended from ObjC-appscript.
This crate is being extracted from `rust-moneymoney`, which needs typed sdef access
to validate its AppleScript bindings against the user's installed MoneyMoney version.
Publishing it as a standalone crate makes the parser available to anyone else in the
Rust ecosystem who needs to introspect a scriptable Mac application's interface.

## References

- **Authoritative DTD** ships with macOS at
  [`/System/Library/DTDs/sdef.dtd`](file:///System/Library/DTDs/sdef.dtd)
  (≈8.6 KB). The DTD is the canonical reference for the AST shape this crate
  models.
- **`sdef(5)` man page**: <https://keith.github.io/xcode-man-pages/sdef.5.html>
- **Apple-provided examples**: `/System/Library/ScriptingDefinitions/` (e.g.
  `CocoaStandard.sdef`, `NSCoreSuite.sdef`).
- **Prior art (Swift)**: [BushelScript SDEFinitely][sdefinitely].

## Implementation choice

`quick-xml` 0.39 with the `serialize` feature, plus hand-written `serde`-deriving
structs that mirror the DTD. Rationale:

- `quick-xml` is the actively-maintained, de-facto standard XML library in Rust;
  ~10× faster than `serde-xml-rs` and substantially better-maintained than
  `yaserde`.
- `serde` derives let us model the DTD's required-vs-optional attribute
  distinction cleanly via `Option<T>` — `yaserde` requires `Default` on every
  type, which is awkward for required attributes.
- Hand-written types are clearer to read and extend than anything an XSD code
  generator could produce, especially since the DTD uses custom entities like
  `(yes | no)` for booleans.

## Crate layout

```
src/
  lib.rs           — crate-level docs, module wiring, Dictionary::from_{str,reader,path}
  dictionary.rs    — Dictionary, Suite, Command
  parameter.rs     — Parameter, DirectParameter, Result_
  yorn.rs          — (yes | no) entity → bool serde helper
  error.rs         — Error type
tests/
  parses.rs        — integration tests against tests/fixtures/
  fixtures/        — synthetic and (eventually) real-world sdef samples
```

## Usage

```rust
use sdef::Dictionary;

let xml = std::fs::read_to_string(
    "/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef",
)?;
let dict: Dictionary = xml.parse()?;

for suite in &dict.suites {
    for command in &suite.commands {
        println!("{} ({})", command.name, command.code);
        for p in &command.parameters {
            println!("  - {} ({}, optional={})", p.name, p.code, p.optional);
        }
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Dictionary` also implements [`FromStr`][fromstr], so the `.parse()` form
above is the preferred entry point. `Dictionary::from_reader` and
`Dictionary::from_path` cover the `Read` and filesystem cases.

[fromstr]: https://doc.rust-lang.org/std/str/trait.FromStr.html

## Scope

**In scope:**
- Parsing the elements `dictionary`, `suite`, `command`, `parameter`,
  `direct-parameter`, `result`.
- Returning a typed, owned AST suitable for inspection by consumers.
- Stable, semver-respecting public API.

**Out of scope (for now):**
- DTD validation of input documents. `quick-xml` does not perform DTD validation;
  this crate trusts the well-formedness of its input. Catching schema drift
  between an sdef and a binding is the caller's job.
- Generating Rust bindings (or any other language's) from an sdef. This crate
  is a parser, not a code generator.
- Writing sdef files (serialisation back to XML).
- Modelling every element the DTD permits. `class`, `enumeration`, `cocoa`,
  `xref`, `synonym`, `record-type`, `value-type`, etc. are on the roadmap but
  not necessarily required for an initial release. Add as consumers need them.

## Roadmap

- [ ] Implement `Dictionary::from_str` properly via `quick_xml::de` (skeleton
      already present in `src/lib.rs`).
- [ ] Cover the remaining DTD elements: `class`, `enumeration`, `record-type`,
      `value-type`, `cocoa`, `xref`, `synonym`.
- [ ] Integration tests against Apple-provided sdefs under
      `/System/Library/ScriptingDefinitions/` (vendoring is fine — those are
      Apple's published samples, not proprietary application content).
- [ ] Document the `(yes | no)` and `(r | w | rw)` DTD entities with custom
      serde deserializers (`yorn`, `rw`).
- [ ] CI matrix: Linux + macOS.
- [ ] Publish 0.1.0 to crates.io.

## License

MIT. See [LICENSE](LICENSE).

[sdef-man]: https://keith.github.io/xcode-man-pages/sdef.5.html
[sdefinitely]: https://github.com/bushelscript/bushelscript
