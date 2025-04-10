use anyhow::{Ok, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

pub(crate) struct Header {
    headers: HeaderMap,
}

impl Header {
    pub(crate) fn builder() -> Self {
        Self {
            headers: HeaderMap::new(),
        }
    }

    pub(crate) fn insert(mut self, key: HeaderName, value: &str) -> Result<Self> {
        let value = HeaderValue::from_str(value)?;
        self.headers.insert(key, value);
        Ok(self)
    }

    pub(crate) fn build(self) -> HeaderMap {
        self.headers
    }
}
