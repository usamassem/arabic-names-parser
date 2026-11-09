# Data attribution

The three CSV files in this directory are vendored, byte-for-byte, from the
[`anazhmetdin/ArabicNamesParser`](https://github.com/anazhmetdin/ArabicNamesParser)
repository, which in turn sources them from:

- **Sultan Qaboos Encyclopedia of Arabic Names**
  (`Sultan_Qaboos_Encyclopedia_Names.csv`) —
  <http://arabiclexicon.hawramani.com/sultan-qaboos-encyclopedia-of-arab-names/>
- **Muslim Baby Names** (`muslimbabynames_hawramani.csv`) —
  <http://muslimbabynames.hawramani.com/>
- `additional_names.csv` — a small hand-curated supplement from the same
  upstream repository.

## License status

As of vendoring, the upstream `ArabicNamesParser` repository declares no
license (GitHub reports `license: null`), and the two scraped source sites
above do not publish explicit redistribution terms either. This data is
embedded here, behind the opt-in `builtin-dict` Cargo feature, as an
explicitly accepted risk — it is not a claim of clear redistribution rights.
If you intend to redistribute a build of this crate with `builtin-dict`
enabled, verify the licensing situation independently first.
