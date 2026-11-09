//! Async reader/writer adapters over `tokio::io::AsyncBufRead`/`AsyncWrite`.
//!
//! [`AsyncNameReader`] hand-rolls [`Stream::poll_next`] against
//! [`tokio::io::AsyncBufRead::poll_fill_buf`]/`consume` rather than boxing a
//! `read_line().await` future, so it stays `Unpin`-friendly without pulling
//! in an extra combinator crate.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use tokio::io::AsyncBufRead;

use crate::NameDictionary;
use crate::engine::NameParser;
use crate::name::ParsedName;

/// A [`Stream`] that reads `R` line by line and parses each line against
/// `dictionary` using `parser`, yielding one [`ParsedName`] per line.
pub struct AsyncNameReader<'p, 'd, R> {
    reader: R,
    parser: &'p NameParser,
    dictionary: &'d NameDictionary,
    buf: Vec<u8>,
}

impl<'p, 'd, R: AsyncBufRead + Unpin> AsyncNameReader<'p, 'd, R> {
    /// Wrap `reader`, parsing each line with `parser` against `dictionary`.
    pub fn new(reader: R, parser: &'p NameParser, dictionary: &'d NameDictionary) -> Self {
        Self {
            reader,
            parser,
            dictionary,
            buf: Vec::new(),
        }
    }

    fn parse_line(&self, raw: Vec<u8>) -> ParsedName {
        let line = String::from_utf8_lossy(&raw);
        self.parser.parse(&line, self.dictionary)
    }
}

impl<'p, 'd, R: AsyncBufRead + Unpin> Stream for AsyncNameReader<'p, 'd, R> {
    type Item = io::Result<ParsedName>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            if let Some(pos) = this.buf.iter().position(|&b| b == b'\n') {
                let mut line: Vec<u8> = this.buf.drain(..=pos).collect();
                line.pop(); // trailing '\n'
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Poll::Ready(Some(Ok(this.parse_line(line))));
            }

            match Pin::new(&mut this.reader).poll_fill_buf(cx) {
                Poll::Ready(Ok(chunk)) => {
                    if chunk.is_empty() {
                        return if this.buf.is_empty() {
                            Poll::Ready(None)
                        } else {
                            let line = std::mem::take(&mut this.buf);
                            Poll::Ready(Some(Ok(this.parse_line(line))))
                        };
                    }
                    let len = chunk.len();
                    this.buf.extend_from_slice(chunk);
                    Pin::new(&mut this.reader).consume(len);
                }
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

/// Write `name` as a single line via its [`std::fmt::Display`] form.
pub async fn write_name_async<W: tokio::io::AsyncWrite + Unpin>(
    writer: &mut W,
    name: &ParsedName,
) -> io::Result<()> {
    use tokio::io::AsyncWriteExt;
    writer.write_all(format!("{name}\n").as_bytes()).await
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use tokio::io::BufReader;

    use super::{AsyncNameReader, write_name_async};
    use crate::{NameDictionary, NameParser};

    #[tokio::test]
    async fn reads_and_parses_each_line_in_order() {
        let input = "محمد\nعلى بن محمد\nمحمد\n".as_bytes();
        let reader = BufReader::new(input);
        let dictionary = NameDictionary::from_normalized_names(["محمد", "على"]);
        let parser = NameParser::default();

        let mut stream = AsyncNameReader::new(reader, &parser, &dictionary);
        let mut results = Vec::new();
        while let Some(item) = stream.next().await {
            results.push(item.unwrap());
        }

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].composing_names_normalized, vec!["محمد"]);
        assert_eq!(
            results[1].composing_names_normalized,
            vec!["على", "بن محمد"]
        );
        assert_eq!(results[2].composing_names_normalized, vec!["محمد"]);
    }

    #[tokio::test]
    async fn handles_a_final_line_with_no_trailing_newline() {
        let input = "محمد".as_bytes();
        let reader = BufReader::new(input);
        let dictionary = NameDictionary::from_normalized_names(["محمد"]);
        let parser = NameParser::default();

        let mut stream = AsyncNameReader::new(reader, &parser, &dictionary);
        let first = stream.next().await.unwrap().unwrap();
        assert_eq!(first.composing_names_normalized, vec!["محمد"]);
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn write_name_async_round_trips_through_display() {
        let dictionary = NameDictionary::from_normalized_names(["محمد"]);
        let parser = NameParser::default();
        let parsed = parser.parse("محمد", &dictionary);

        let mut buf = Vec::new();
        write_name_async(&mut buf, &parsed).await.unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), format!("{parsed}\n"));
    }
}
