//! Parse an sdef file from a path and print a one-line-per-term summary.
//!
//! Usage:
//!
//! ```sh
//! cargo run --example dump -- /Applications/Some.app/Contents/Resources/Some.sdef
//! ```
//!
//! Useful as a smoke-test of the public API on a real sdef and as a
//! starting point for downstream tools.

use std::process::ExitCode;

use sdef::Dictionary;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: dump <path/to/file.sdef>");
        return ExitCode::from(2);
    };

    let dict = match Dictionary::from_path(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };

    if let Some(title) = &dict.title {
        println!("# {title}");
    }

    for suite in &dict.suites {
        println!("\n## suite {} ({})", suite.name, suite.code);

        for cmd in &suite.commands {
            println!("  command   {} ({})", cmd.name, cmd.code);
        }
        for event in &suite.events {
            println!("  event     {} ({})", event.name, event.code);
        }
        for class in &suite.classes {
            println!("  class     {} ({})", class.name, class.code);
        }
        for ext in &suite.class_extensions {
            println!("  extends   {}", ext.extends);
        }
        for en in &suite.enumerations {
            println!(
                "  enum      {} ({}) [{} variants]",
                en.name,
                en.code,
                en.enumerators.len()
            );
        }
        for rt in &suite.record_types {
            println!(
                "  record    {} ({}) [{} props]",
                rt.name,
                rt.code,
                rt.properties.len()
            );
        }
        for vt in &suite.value_types {
            println!("  valuetype {} ({})", vt.name, vt.code);
        }
    }

    ExitCode::SUCCESS
}
