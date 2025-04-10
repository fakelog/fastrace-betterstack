mod header;

use std::borrow::Cow;

use anyhow::{Context, Result};
use fastrace::collector::Reporter;
use fastrace::prelude::SpanRecord;
use header::Header;
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use rmp_serde::Serializer;
use serde::Serialize;

#[derive(Serialize)]
pub struct BetterstackMessage<'a> {
    pub source: Cow<'a, str>,
    pub message: Cow<'a, str>,
    pub level: Cow<'a, str>,
}

pub struct BetterstackReporter {
    ingest_host: String,
    token: String,
    client: Client,
}

impl BetterstackReporter {
    pub fn new(ingest_host: impl Into<String>, token: impl Into<String>) -> Self {
        BetterstackReporter {
            ingest_host: ingest_host.into(),
            token: token.into(),
            client: Client::new(),
        }
    }

    fn convert<'a>(&self, spans: &'a [SpanRecord]) -> Vec<BetterstackMessage<'a>> {
        spans
            .iter()
            .flat_map(|span| {
                span.events.iter().map(|event| {
                    let level = event
                        .properties
                        .iter()
                        .find(|(key, _)| *key == "level")
                        .map_or("INFO", |(_, val)| val);

                    BetterstackMessage {
                        source: Cow::Borrowed(&span.name),
                        message: Cow::Borrowed(&event.name),
                        level: Cow::Borrowed(level),
                    }
                })
            })
            .collect()
    }

    fn serialize(&self, spans: Vec<BetterstackMessage<'_>>) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        spans
            .serialize(&mut Serializer::new(&mut buf).with_struct_map())
            .context("Failed to serialize messages")?;

        Ok(buf)
    }

    fn try_report(&self, spans: &[SpanRecord]) -> Result<()> {
        let messages = self.convert(spans);
        let bytes = self.serialize(messages)?;

        let headers = Header::builder()
            .insert(AUTHORIZATION, &format!("Bearer {}", self.token))
            .context("Invalid authorization header")?
            .insert(CONTENT_TYPE, "application/msgpack")
            .context("Invalid content type header")?
            .build();

        let response = self
            .client
            .post(&self.ingest_host)
            .headers(headers)
            .body(bytes)
            .send()
            .context("Failed to send request to Better Stack")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().unwrap_or_else(|_| "<empty>".into());
            anyhow::bail!(
                "Better Stack API error: status={}, body={}",
                status,
                error_body
            );
        }

        Ok(())
    }
}

impl Reporter for BetterstackReporter {
    fn report(&mut self, spans: Vec<SpanRecord>) {
        if spans.is_empty() {
            return;
        }

        if let Err(err) = self.try_report(&spans) {
            log::error!("Failed to report to Better Stack: {:#}", err);
        }
    }
}
