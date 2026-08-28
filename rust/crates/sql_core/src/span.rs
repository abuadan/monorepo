use std::ops::Range;

pub type ByteOffset = usize;
pub type ByteSpan = Range<ByteOffset>;
pub type Spanned<T> = (T, ByteSpan);

#[derive(Debug, Clone)]
pub struct SourceMap<'a> {
    source: &'a str,
    line_starts: Vec<usize>,
}

impl<'a> SourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut line_starts = vec![0];
        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push(idx + ch.len_utf8());
            }
        }
        Self {
            source,
            line_starts,
        }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        let column = self.source[line_start..offset].chars().count() + 1;
        (line_index + 1, column)
    }
}
