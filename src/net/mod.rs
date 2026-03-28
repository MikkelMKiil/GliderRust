use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone)]
pub struct NetClient {
    client: reqwest::Client,
}

impl Default for NetClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl NetClient {
    pub async fn health_check(&self, url: &str) -> Result<u16, NetError> {
        let response = self.client.get(url).send().await?;
        Ok(response.status().as_u16())
    }
}
