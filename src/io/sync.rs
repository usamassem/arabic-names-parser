//! Synchronous reader/writer adapters over [`std::io::BufRead`]/[`std::io::Write`].

use std::io::{self, BufRead, Write};

use crate::NameDictionary;
use crate::engine::NameParser;
use crate::name::ParsedName;

/// An [`Iterator`] that reads `R` line by line and parses each line against
/// `dictionary` using `parser`, yielding one [`ParsedName`] per line.
pub struct NameReader<'p, 'd, R> {
    reader: R,
    parser: &'p NameParser,
    dictionary: &'d NameDictionary,
}

impl<'p, 'd, R: BufRead> NameReader<'p, 'd, R> {
    /// Wrap `reader`, parsing each line with `parser` against `dictionary`.
    pub fn new(reader: R, parser: &'p NameParser, dictionary: &'d NameDictionary) -> Self {
        Self {
            reader,
            parser,
            dictionary,
        }
    }
}

impl<'p, 'd, R: BufRead> Iterator for NameReader<'p, 'd, R> {
    type Item = io::Result<ParsedName>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                let line = line.trim_end_matches(['\n', '\r']);
                Some(Ok(self.parser.parse(line, self.dictionary)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// Write `name` as a single line via its [`std::fmt::Display`] form.
pub fn write_name<W: Write>(writer: &mut W, name: &ParsedName) -> io::Result<()> {
    writeln!(writer, "{name}")
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};

    use super::{NameReader, write_name};
    use crate::{NameDictionary, NameParser};

    #[test]
    fn reads_and_parses_each_line_in_order() {
        let input = "محمد\nعلى بن محمد\nمحمد\n";
        let reader = BufReader::new(Cursor::new(input));
        let dictionary = NameDictionary::from_normalized_names(["محمد", "على"]);
        let parser = NameParser::default();

        let results: Vec<_> = NameReader::new(reader, &parser, &dictionary)
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].composing_names_normalized, vec!["محمد"]);
        assert_eq!(
            results[1].composing_names_normalized,
            vec!["على", "بن محمد"]
        );
        assert_eq!(results[2].composing_names_normalized, vec!["محمد"]);
    }

    #[test]
    fn write_name_round_trips_through_display() {
        let dictionary = NameDictionary::from_normalized_names(["محمد"]);
        let parser = NameParser::default();
        let parsed = parser.parse("محمد", &dictionary);

        let mut buf = Vec::new();
        write_name(&mut buf, &parsed).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), format!("{parsed}\n"));
    }
}
