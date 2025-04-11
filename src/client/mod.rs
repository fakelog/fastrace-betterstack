mod header;

use anyhow::{Context, Ok, Result};
use reqwest::{
    blocking::{Client, Response},
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap},
};
use std::sync::Arc;

use header::Header;

pub struct BetterStackClient {
    reqwest_client: Client,
    token: Arc<str>,
    url: Arc<str>,
}

impl BetterStackClient {
    pub(crate) fn new(url: Arc<str>, token: Arc<str>) -> Self {
        BetterStackClient {
            reqwest_client: Client::new(),
            token,
            url,
        }
    }

    fn get_headers(&self) -> Result<HeaderMap> {
        let headers = Header::builder()
            .insert(AUTHORIZATION, &format!("Bearer {}", &self.token))
            .context("Invalid authorization header")?
            .insert(CONTENT_TYPE, "application/msgpack")
            .context("Invalid content type header")?
            .build();

        Ok(headers)
    }

    pub(crate) fn send_message(&self, message: Vec<u8>) -> Result<Response> {
        let headers = self.get_headers()?;

        let response = self
            .reqwest_client
            .post(self.url.as_ref())
            .headers(headers)
            .body(message)
            .send()
            .context("Failed to send request to Better Stack")?;

        Ok(response)
    }
}
