use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::span::ByteSpan;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LexError {
    #[error("unexpected token at bytes {span:?}")]
    InvalidToken { span: ByteSpan },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticLabel {
    pub span: ByteSpan,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("expected {expected} at bytes {span:?}")]
    Expected {
        expected: &'static str,
        span: ByteSpan,
        labels: Vec<DiagnosticLabel>,
    },
    #[error("unexpected token {found:?} at bytes {span:?}")]
    UnexpectedToken {
        found: String,
        span: ByteSpan,
        labels: Vec<DiagnosticLabel>,
    },
    #[error("unexpected end of input")]
    Eof,
}

impl ParseError {
    pub fn labels(&self) -> &[DiagnosticLabel] {
        match self {
            Self::Expected { labels, .. } | Self::UnexpectedToken { labels, .. } => labels,
            Self::Eof => &[],
        }
    }

    pub fn render(&self, source_name: &str, source: &str) -> String {
        let mut out = Vec::new();
        self.write(source_name, source, &mut out)
            .expect("diagnostic writes to a byte buffer");
        String::from_utf8(out).expect("ariadne diagnostics are valid utf-8")
    }

    pub fn write(
        &self,
        source_name: &str,
        source: &str,
        mut writer: impl std::io::Write,
    ) -> std::io::Result<()> {
        let primary_span = match self {
            Self::Expected { span, .. } | Self::UnexpectedToken { span, .. } => span.clone(),
            Self::Eof => source.len()..source.len(),
        };

        let message = self.to_string();
        let mut report = Report::build(ReportKind::Error, (source_name, primary_span.clone()))
            .with_message(message.clone())
            .with_label(
                Label::new((source_name, primary_span))
                    .with_message(message)
                    .with_color(Color::Red),
            );

        for label in self.labels() {
            report = report.with_label(
                Label::new((source_name, label.span.clone()))
                    .with_message(label.message.clone())
                    .with_color(Color::Yellow),
            );
        }

        report
            .finish()
            .write((source_name, Source::from(source)), &mut writer)
    }
}
