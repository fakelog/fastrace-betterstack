pub mod appender;
mod client;

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::{Context, Ok, Result};
use client::BetterStackClient;
use fastrace::collector::Reporter;
use fastrace::prelude::SpanRecord;
use rmp_serde::Serializer;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct BetterstackSpan<'a> {
    pub source: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub level: Cow<'a, str>,
}

pub struct BetterstackReporter {
    ingest_host: Arc<str>,
    token: Arc<str>,
}

impl BetterstackReporter {
    pub fn new(ingest_host: impl Into<Arc<str>>, token: impl Into<Arc<str>>) -> Self {
        BetterstackReporter {
            ingest_host: ingest_host.into(),
            token: token.into(),
        }
    }

    fn convert<'a>(&self, spans: &'a [SpanRecord]) -> Vec<BetterstackSpan<'a>> {
        spans
            .iter()
            .flat_map(|span| {
                span.events.iter().map(|event| {
                    let level = event
                        .properties
                        .iter()
                        .find(|(key, _)| *key == "level")
                        .map_or("INFO", |(_, val)| val);

                    BetterstackSpan {
                        source: Cow::Borrowed(&span.name),
                        message: Cow::Borrowed(&event.name),
                        level: Cow::Borrowed(level),
                    }
                })
            })
            .collect()
    }

    fn serialize(&self, spans: Vec<BetterstackSpan<'_>>) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        spans
            .serialize(&mut Serializer::new(&mut buf).with_struct_map())
            .context("Failed to serialize messages")?;

        Ok(buf)
    }

    fn try_report(&self, spans: Vec<SpanRecord>) -> Result<()> {
        let spans = self.convert(&spans);
        let bytes = self.serialize(spans)?;

        let client = BetterStackClient::new(self.ingest_host.clone(), self.token.clone());
        let _ = client.send_message(bytes)?;

        Ok(())
    }
}

impl Reporter for BetterstackReporter {
    fn report(&mut self, spans: Vec<SpanRecord>) {
        if spans.is_empty() {
            return;
        }

        if let Err(err) = self.try_report(spans) {
            log::error!("Failed to report to Better Stack: {:#}", err);
        }
    }
}
