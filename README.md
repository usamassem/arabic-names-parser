# arabic-names-parser

A low-level, I/O-agnostic engine for normalizing and decomposing Arabic
personal names into prefix, known-name, and postfix components.

This is a Rust port of the algorithm in
[`anazhmetdin/ArabicNamesParser`](https://github.com/anazhmetdin/ArabicNamesParser),
restructured as a pure, synchronous core (`NameParser::parse`) that takes a
`&str` and a caller-supplied `NameDictionary` and returns a `ParsedName` —
no I/O baked in, so it drops into any sync or async reader/writer. Optional
features add first-party sync/async I/O adapters, an embedded default
dictionary, and a pluggable online-lookup extension point.

## Example

```rust
use arabic_names_parser::{normalize_arabic_name, NameDictionary, NameParser};

let dictionary = NameDictionary::from_raw_names(
    ["على", "عبد الله", "محمد"],
    normalize_arabic_name,
);
let parser = NameParser::default();
let parsed = parser.parse("ابن علي [Ibn Ali]", &dictionary);

assert_eq!(parsed.other_info, vec!["Ibn Ali"]);
assert_eq!(parsed.full_name_normalized, "ابن على");
assert_eq!(parsed.composing_names_normalized, vec!["ابن على"]);
```

## Features

| Feature         | Adds                                                                 |
|------------------|-----------------------------------------------------------------------|
| *(none)*         | The core engine only — bring your own `NameDictionary`.             |
| `builtin-dict`   | An embedded default dictionary sourced from three reference datasets. |
| `sync-io`        | `NameReader` (a `BufRead`-driven iterator) and a line writer.       |
| `async-io`       | The async equivalents over `tokio::io::AsyncBufRead`/`AsyncWrite`.   |
| `online-lookup`  | `OnlineNameSource`, a pluggable trait for augmenting the dictionary from an external source, kept out of the synchronous hot path. |

## Documented deviations from upstream

Tracing the real Python runtime behavior of the reference library (rather
than trusting its README) turned up two places where this port
deliberately diverges, both of which are configurable:

1. **Adjacency merging.** Upstream's README claims a prefix/postfix word
   swallowed next to a matched known name stays as its own output token.
   The real code merges it into the matched name's single token instead.
   This crate reproduces the real behavior by default
   (`AdjacencyMode::Merge`) and offers the README's documented behavior as
   an opt-in (`AdjacencyMode::Separate`).
2. **The compound fix-pattern phase.** Upstream's equivalent phase is dead
   code at runtime due to a string-escaping bug in how its regex patterns
   are built. This crate implements the *intended, working* version
   instead.

See the crate's top-level documentation (`cargo doc --open`) for details.

## Data attribution and licensing

The `builtin-dict` feature embeds three datasets vendored from the
reference repository — see [`data/ATTRIBUTION.md`](data/ATTRIBUTION.md)
for sources and a licensing caveat: the upstream repository declares no
license, and the two scraped source sites do not publish explicit
redistribution terms either. This data is embedded as an explicitly
accepted risk when `builtin-dict` is enabled; the pure engine (default
feature set) carries no such risk since it ships no data at all.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)).
