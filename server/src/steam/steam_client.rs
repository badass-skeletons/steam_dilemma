use log::warn;
use reqwest::Client;
use reqwest::Error;
use reqwest::StatusCode;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

// Possible SteamID's
// SteamID - STEAM_0:0:11101
// SteamID3 - [U:1:22202]
// SteamID3 without brackets - U:1:22202
// SteamID64 - 76561197960287930
// CustomURL - gabelogannewell
// Full Steam URL - https://steamcommunity.com/profiles/76561197960287930
// Full Steam URL with customURL - https://steamcommunity.com/id/gabelogannewell

/// Represents an error that was returned by a Steam API endpoint.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SteamError {
    /// A reqwest failed for some reason
    #[error("Error response from steam: {0}")]
    FailedRequest(String),
    /// The requested data is either private, or not present at all. Usually, this comes from a
    /// deserialization error in serde.
    #[error("The data you requested is either private or empty")]
    NoData,
}

impl From<reqwest::Error> for SteamError {
    fn from(err: Error) -> Self {
        // If the reqwest goes wrong, we should forward it to the user
        let reqwest_error = err.to_string();
        Self::FailedRequest(reqwest_error)
    }
}

/// This struct holds the blocking reqwest client and is used to interact with the API.
pub struct SteamClient {
    client: Client,
    api_key: String,
}

impl Default for SteamClient {
    fn default() -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_key: String::new(),
        }
    }
}

impl SteamClient {
    /// Returns a new SteamClient instance carrying a developer API token
    pub fn from(api_key: String) -> Self {
        let client = reqwest::Client::new();
        Self { client, api_key }
    }

    /// Return a SteamClient without a Steam API token
    pub fn new() -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_key: String::new(),
        }
    }

    pub async fn get_request<T: Serialize>(
        &self,
        endpoint: &str,
        query: Vec<(&str, T)>,
    ) -> Result<Value, SteamError> {
        if self.api_key.is_empty() {
            warn!("Not using a valid API key. Is this on purpose?")
        }
        let request = self
            .client
            .get(endpoint)
            .query(&[("key", self.api_key.clone())])
            .query(&query)
            .build()?;

        log::debug!("New request : {:?}", request);

        let response = self.client.execute(request);

        match response.await {
            Ok(r) => match r.status() {
                StatusCode::OK => Ok(r.json().await.unwrap()),  // we trust steam that we'll actually get json w/ a 200 response, so unwrap() is good enough
                StatusCode::UNAUTHORIZED => {
                    Err(SteamError::FailedRequest("Unauthorized. Either you have used an invalid API key, or the data you wanted to access is private".to_string()))
                }
                _ => Err(SteamError::FailedRequest(
                    "Steam could not process your request. Double-check your provided parameters (Steam ID, app ID, ...).".to_string(),
                )),
            },
            Err(_) => Err(SteamError::FailedRequest(
                "Something went wrong with your request".to_string(),
            )),
        }
    }

    pub fn parse_response<R: DeserializeOwned, S: From<R> + DeserializeOwned>(
        &self,
        response: Value,
    ) -> Result<S, SteamError> {
        let res = serde_json::from_value::<R>(response);
        if let Ok(v) = res {
            Ok(v.into())
        } else {
            Err(SteamError::NoData)
        }
    }
}
