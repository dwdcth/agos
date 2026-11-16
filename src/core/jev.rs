//! Jev System-One decision protocol types and HTTP client.
//!
//! Compatible with both `pa-jev` (`POST /v1/systemone`, `POST /decisions`) and
//! standard System-One Jev requests (accepting `choices`/`levels` aliases and
//! string or JSON-object `state`).

use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Deserialize `state` from either a plain string or an arbitrary JSON object/value.
fn deserialize_state_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::String(s) => Ok(s),
        other => serde_json::to_string(&other).map_err(serde::de::Error::custom),
    }
}

/// Structured decision request: evaluate `questions` against `state`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(deserialize_with = "deserialize_state_string")]
    pub state: String,
    pub questions: BTreeMap<String, Question>,
}

/// Supported Jev System-One question primitives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Question {
    Choice {
        instructions: String,
        #[serde(alias = "choices")]
        criteria: BTreeMap<String, String>,
    },
    Score {
        instructions: String,
        #[serde(alias = "levels")]
        criteria: Vec<String>,
    },
    Noul {
        instructions: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        criteria: Option<BTreeMap<String, String>>,
    },
}

/// Decision response returned by `/v1/systemone` or `/decisions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JevResponse {
    pub model: String,
    pub answers: BTreeMap<String, Answer>,
    #[serde(default)]
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Choice {
        choice: String,
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
    Score {
        score: f64,
        probabilities: BTreeMap<String, f64>,
        confidence: f64,
    },
    Noul {
        noul: f64,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    pub inputs: u64,
    pub tokens_evaluated: u64,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JevClientConfig {
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Error)]
pub enum JevClientError {
    #[error("invalid jev config: {0}")]
    Config(String),
    #[error("jev http transport error: {0}")]
    Transport(String),
    #[error("jev server returned {status}: {body}")]
    HttpStatus { status: u16, body: String },
    #[error("invalid jev response payload: {0}")]
    Decode(String),
}

/// Normalize a gateway URL to its root origin/path without trailing `/` or `/v1`.
pub(crate) fn normalize_gateway_root(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    trimmed.strip_suffix("/v1").unwrap_or(trimmed).to_string()
}

/// Async + sync-bridged HTTP client for Jev-compatible model gateway services.
#[derive(Debug, Clone)]
pub struct JevHttpClient {
    client: reqwest::Client,
    root_url: String,
    api_key: Option<String>,
    model: Option<String>,
}

impl JevHttpClient {
    pub fn new(config: &JevClientConfig) -> Result<Self, JevClientError> {
        let root_url = normalize_gateway_root(&config.base_url);
        if root_url.is_empty() {
            return Err(JevClientError::Config("base_url must not be empty".into()));
        }
        let timeout = Duration::from_secs(config.timeout_seconds.unwrap_or(30).max(1));
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| JevClientError::Config(format!("reqwest client build: {e}")))?;
        Ok(Self {
            client,
            root_url,
            api_key: config
                .api_key
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
            model: config
                .model
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string),
        })
    }

    pub fn root_url(&self) -> &str {
        &self.root_url
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(k) => req.bearer_auth(k),
            None => req,
        }
    }

    pub async fn evaluate(&self, req: &JevRequest) -> Result<JevResponse, JevClientError> {
        let mut payload = req.clone();
        if payload.model.is_none() {
            payload.model = self.model.clone();
        }
        let url = format!("{}/v1/systemone", self.root_url);
        let resp = self
            .apply_auth(self.client.post(&url).json(&payload))
            .send()
            .await
            .map_err(|e| JevClientError::Transport(format!("POST {url}: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(JevClientError::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }
        resp.json::<JevResponse>()
            .await
            .map_err(|e| JevClientError::Decode(format!("decode JevResponse: {e}")))
    }

    pub async fn generate_short(
        &self,
        system: &str,
        user: &str,
        max_tokens: u32,
        json_object: bool,
    ) -> Result<String, JevClientError> {
        let url = format!("{}/v1/chat/completions", self.root_url);
        let mut body = serde_json::json!({
            "model": self.model.as_deref().unwrap_or("local"),
            "temperature": 0.0,
            "max_tokens": max_tokens,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user }
            ]
        });
        if json_object {
            body["response_format"] = serde_json::json!({ "type": "json_object" });
        }
        let resp = self
            .apply_auth(self.client.post(&url).json(&body))
            .send()
            .await
            .map_err(|e| JevClientError::Transport(format!("POST {url}: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let err_body = resp.text().await.unwrap_or_default();
            return Err(JevClientError::HttpStatus {
                status: status.as_u16(),
                body: err_body,
            });
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| JevClientError::Decode(format!("decode chat completion: {e}")))?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                JevClientError::Decode(format!("missing choices[0].message.content in {json}"))
            })?;
        Ok(content.to_string())
    }

    pub async fn healthz(&self) -> Result<bool, JevClientError> {
        let url = format!("{}/healthz", self.root_url);
        let resp = self
            .apply_auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| JevClientError::Transport(format!("GET {url}: {e}")))?;
        Ok(resp.status().is_success())
    }

    pub async fn list_models(&self) -> Result<Vec<String>, JevClientError> {
        let url = format!("{}/v1/models", self.root_url);
        let resp = self
            .apply_auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| JevClientError::Transport(format!("GET {url}: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(JevClientError::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }
        let val: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| JevClientError::Decode(format!("decode models list: {e}")))?;
        let ids = val["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item["id"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ids)
    }

    pub fn evaluate_blocking(&self, req: &JevRequest) -> Result<JevResponse, JevClientError> {
        block_on_jev(self.evaluate(req))
    }

    pub fn generate_short_blocking(
        &self,
        system: &str,
        user: &str,
        max_tokens: u32,
        json_object: bool,
    ) -> Result<String, JevClientError> {
        block_on_jev(self.generate_short(system, user, max_tokens, json_object))
    }
}

fn block_on_jev<F, T>(fut: F) -> Result<T, JevClientError>
where
    F: std::future::Future<Output = Result<T, JevClientError>>,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(fut))
        }
        _ => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| JevClientError::Transport(format!("tokio runtime: {e}")))?;
            rt.block_on(fut)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_jev_aliases_and_object_state() {
        let raw = r#"{
          "model": "typesafe/jev-1.13",
          "state": { "customer_message": "Order arrived broken." },
          "questions": {
            "is_refund": { "type": "noul", "instructions": "Refund requested?" },
            "dept": {
              "type": "choice",
              "instructions": "Which dept?",
              "choices": { "billing": "Money", "returns": "Broken items" }
            },
            "frustration": {
              "type": "score",
              "instructions": "Level?",
              "levels": ["Calm", "Annoyed"]
            }
          }
        }"#;
        let req: JevRequest = serde_json::from_str(raw).expect("parse standard Jev payload");
        assert!(req.state.contains("customer_message"));
        assert_eq!(req.questions.len(), 3);
    }

    #[test]
    fn normalizes_gateway_root_url() {
        assert_eq!(
            normalize_gateway_root("http://127.0.0.1:8081/v1/"),
            "http://127.0.0.1:8081"
        );
        assert_eq!(
            normalize_gateway_root("http://127.0.0.1:8081/"),
            "http://127.0.0.1:8081"
        );
    }

    #[test]
    fn decodes_typed_and_untagged_noul_answer() {
        let raw = r#"{
          "model": "m",
          "answers": {
            "a": { "noul": 0.91 },
            "b": { "type": "noul", "noul": 0.42 }
          }
        }"#;
        let resp: JevResponse = serde_json::from_str(raw).expect("decode JevResponse");
        assert!(matches!(resp.answers["a"], Answer::Noul { noul } if (noul - 0.91).abs() < 1e-6));
        assert!(matches!(resp.answers["b"], Answer::Noul { noul } if (noul - 0.42).abs() < 1e-6));
    }
}
