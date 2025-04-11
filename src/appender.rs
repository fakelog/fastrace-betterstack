use std::{borrow::Cow, sync::Arc};

use anyhow::{Context, Result};
use logforth::{Append, Diagnostic};
use rmp_serde::Serializer;
use serde::Serialize;

use crate::client::BetterStackClient;

#[derive(Serialize, Debug)]
pub struct BetterstackMessage {
    pub source: Cow<'static, str>,
    pub message: Cow<'static, str>,
    pub level: Cow<'static, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Cow<'static, str>>,
}

#[derive(Debug)]
pub struct BetterStackAppender {
    ingest_host: Arc<str>,
    token: Arc<str>,
    source: Arc<str>,
}

impl BetterStackAppender {
    pub fn new(
        ingest_host: impl Into<Arc<str>>,
        token: impl Into<Arc<str>>,
        source: impl Into<Arc<str>>,
    ) -> Self {
        BetterStackAppender {
            ingest_host: ingest_host.into(),
            token: token.into(),
            source: source.into(),
        }
    }

    fn convert_record(
        &self,
        record: &log::Record,
        diagnostics: &[Box<dyn Diagnostic>],
    ) -> BetterstackMessage {
        let diagnostics = if diagnostics.is_empty() {
            None
        } else {
            Some(Cow::Owned(format!("{:?}", diagnostics)))
        };

        BetterstackMessage {
            source: Cow::Owned(self.source.to_string()),
            message: Cow::Owned(record.args().to_string()),
            level: Cow::Owned(record.level().as_str().to_string()),
            diagnostics,
        }
    }

    fn serialize(&self, message: BetterstackMessage) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        message
            .serialize(&mut Serializer::new(&mut buf).with_struct_map())
            .context("Failed to serialize Betterstack message")?;

        Ok(buf)
    }
}

impl Append for BetterStackAppender {
    fn append(
        &self,
        record: &log::Record,
        diagnostics: &[Box<dyn logforth::Diagnostic>],
    ) -> Result<()> {
        let message = self.convert_record(record, diagnostics);
        let bytes = self.serialize(message)?;

        let client = BetterStackClient::new(self.ingest_host.clone(), self.token.clone());

        client
            .send_message(bytes)
            .context("Failed to send log to Better Stack")?;

        Ok(())
    }
}
