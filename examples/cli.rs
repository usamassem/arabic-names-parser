//! Reads Arabic names from stdin, one per line, and prints each parsed
//! result against the embedded builtin dictionary.
//!
//! Run with: `cargo run --example cli --features builtin-dict,sync-io`

use std::io::{self, Write};

use arabic_names_parser::NameParser;
use arabic_names_parser::builtin::builtin_dictionary;
use arabic_names_parser::io::sync::NameReader;

fn main() -> io::Result<()> {
    let dictionary = builtin_dictionary();
    let parser = NameParser::default();
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    for parsed in NameReader::new(stdin, &parser, dictionary) {
        writeln!(stdout, "{}", parsed?)?;
    }

    Ok(())
}
