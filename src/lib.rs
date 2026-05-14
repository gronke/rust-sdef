//! Typed parser for Apple [scripting definition][sdef-man] (`.sdef`) XML files.
//!
//! Apple's scriptable macOS applications describe their AppleScript-visible
//! commands, classes, and enumerations in an XML file ending in `.sdef`. This
//! crate parses such files into a typed AST suitable for programmatic
//! inspection — e.g. validating Rust bindings against an installed app's
//! actual interface, generating documentation, or building richer
//! AppleScript-driven tools.
//!
//! The authoritative schema is shipped with macOS at
//! `/System/Library/DTDs/sdef.dtd`; this crate's AST mirrors the subset of
//! elements currently modelled. See the [README] for scope and roadmap.
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
//! [sdef-man]: https://keith.github.io/xcode-man-pages/sdef.5.html
//! [README]: https://github.com/<owner>/rust-sdef#readme

#![warn(missing_docs)]

mod dictionary;
mod error;
mod metadata;
mod parameter;
mod yorn;

pub use dictionary::{Command, Dictionary, Suite};
pub use error::Error;
pub use metadata::{AccessGroup, Cocoa};
pub use parameter::{DirectParameter, Parameter, Result_};

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

    /// Find a command by its human-readable `name` attribute, searching all
    /// suites. Returns `None` if no command with that name exists.
    pub fn command(&self, name: &str) -> Option<&Command> {
        self.suites
            .iter()
            .flat_map(|s| &s.commands)
            .find(|c| c.name == name)
    }
}
