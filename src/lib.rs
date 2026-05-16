//! Typed parser for Apple [scripting definition][sdef-man] (`.sdef`) XML files.
//!
//! Apple's scriptable macOS applications describe their AppleScript-visible
//! commands, classes, and enumerations in an XML file ending in `.sdef`. This
//! crate parses such files into a typed AST suitable for programmatic
//! inspection — e.g. validating Rust bindings against an installed app's
//! actual interface, generating documentation, or building richer
//! AppleScript-driven tools.
//!
//! The AST covers every element defined in `/System/Library/DTDs/sdef.dtd`
//! as of macOS 26.x. See the [README] for the per-OS-release change history
//! the AST is annotated against.
//!
//! # Lenient vs strict parsing
//!
//! [`Dictionary::from_str`] (and its `from_reader`/`from_path` siblings)
//! tolerates unknown XML elements: anything outside the modelled DTD subset
//! is silently dropped, which matches quick-xml's default serde behaviour.
//! Use this for forward compatibility against future Apple additions.
//!
//! [`Dictionary::from_str_strict`] (and friends) runs a pre-pass that
//! rejects any element name this crate does not model and any
//! `<xi:include>` directive. Use it when you want loud failures on DTD
//! drift — for tests, CI gates, or before pinning a fixture for review.
//!
//! # Example
//!
//! ```no_run
//! use sdef::Dictionary;
//!
//! let xml = std::fs::read_to_string(
//!     "/Applications/MoneyMoney.app/Contents/Resources/MoneyMoney.sdef",
//! )?;
//! let dict: Dictionary = xml.parse()?;
//!
//! for suite in &dict.suites {
//!     for command in &suite.commands {
//!         println!("{} ({})", command.name, command.code);
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Strict mode surfaces DTD drift via typed error variants:
//!
//! ```
//! use sdef::{Dictionary, Error};
//!
//! let xml = r#"<?xml version="1.0"?>
//! <dictionary>
//!     <suite name="X" code="SUIT">
//!         <command name="c" code="SUITcmd1">
//!             <vendor-extension/>
//!         </command>
//!     </suite>
//! </dictionary>"#;
//!
//! match Dictionary::from_str_strict(xml) {
//!     Err(Error::UnknownElement { name }) => assert_eq!(name, "vendor-extension"),
//!     other => panic!("expected UnknownElement, got {other:?}"),
//! }
//! ```
//!
//! [sdef-man]: https://keith.github.io/xcode-man-pages/sdef.5.html
//! [README]: https://github.com/gronke/rust-sdef#readme

#![warn(missing_docs)]

mod class;
mod command;
mod decl;
mod dictionary;
mod error;
mod metadata;
mod strict;
mod typeref;
mod yorn;

pub use class::{Accessor, AccessorStyle, Class, ClassExtension, Contents, Element, RespondsTo};
pub use command::{Command, CommandResult, DirectParameter, Event, Parameter};
pub use decl::{Enumeration, Enumerator, Property, RecordType, ValueType};
pub use dictionary::{Dictionary, Suite};
pub use error::Error;
pub use metadata::{AccessGroup, Cocoa, Documentation, Synonym, Xref};
pub use typeref::TypeRef;

use std::io::Read;
use std::path::Path;
use std::str::FromStr;

impl FromStr for Dictionary {
    type Err = Error;

    fn from_str(xml: &str) -> Result<Self, Self::Err> {
        quick_xml::de::from_str(xml).map_err(Error::from)
    }
}

impl Dictionary {
    /// Parse an sdef document from any [`std::io::Read`] source.
    pub fn from_reader<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        buf.parse()
    }

    /// Parse an sdef document from a filesystem path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        std::fs::read_to_string(path)?.parse()
    }

    /// Parse an sdef document and reject anything outside the modelled DTD
    /// subset. See the [crate-level docs] for the lenient/strict contrast.
    ///
    /// Returns [`Error::UnknownElement`] for unmodelled element names and
    /// [`Error::XIncludeUnsupported`] for `<xi:include>` directives.
    ///
    /// [crate-level docs]: crate
    pub fn from_str_strict(xml: &str) -> Result<Self, Error> {
        strict::validate_strict(xml)?;
        xml.parse()
    }

    /// Strict-mode counterpart to [`Self::from_reader`].
    pub fn from_reader_strict<R: Read>(mut reader: R) -> Result<Self, Error> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Self::from_str_strict(&buf)
    }

    /// Strict-mode counterpart to [`Self::from_path`].
    pub fn from_path_strict<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let xml = std::fs::read_to_string(path)?;
        Self::from_str_strict(&xml)
    }

    /// Find a command by its human-readable `name` attribute, searching all
    /// suites. Returns `None` if no command with that name exists.
    pub fn command(&self, name: &str) -> Option<&Command> {
        self.suites
            .iter()
            .flat_map(|s| &s.commands)
            .find(|c| c.name == name)
    }

    /// Find a class by its human-readable `name` attribute, searching all
    /// suites. Returns `None` if no class with that name exists.
    pub fn class(&self, name: &str) -> Option<&Class> {
        self.suites
            .iter()
            .flat_map(|s| &s.classes)
            .find(|c| c.name == name)
    }

    /// Find a suite by its `name` attribute. Returns `None` if no suite with
    /// that name exists.
    pub fn suite(&self, name: &str) -> Option<&Suite> {
        self.suites.iter().find(|s| s.name == name)
    }

    /// Find an enumeration by its `name` attribute, searching all suites.
    /// Returns `None` if no enumeration with that name exists.
    pub fn enumeration(&self, name: &str) -> Option<&Enumeration> {
        self.suites
            .iter()
            .flat_map(|s| &s.enumerations)
            .find(|e| e.name == name)
    }
}
