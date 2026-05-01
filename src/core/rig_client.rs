use bytes::Bytes;
use rig::http_client::{
    Error as HttpError, HttpClientExt, LazyBody, MultipartForm, Response as HttpResponse,
    Result as HttpResult, StreamingResponse,
};
use rig::providers::openai;
use std::sync::{Arc, Mutex};

/// Custom HTTP client for OpenAI-compatible APIs.
///
/// Handles provider quirks at the HTTP response level:
/// - Fixes empty `role` (some APIs return `""` instead of `"assistant"`)
/// - Extracts `reasoning_content` (Qwen/DeepSeek thinking mode)
#[derive(Clone, Debug, Default)]
pub struct CustomClient {
    inner: reqwest::Client,
    last_reasoning: Arc<Mutex<Option<String>>>,
}

impl CustomClient {
    pub fn new(inner: reqwest::Client) -> Self {
        Self {
            inner,
            last_reasoning: Arc::new(Mutex::new(None)),
        }
    }

    /// Take the reasoning_content from the last response (if any).
    pub fn take_reasoning(&self) -> Option<String> {
        self.last_reasoning.lock().unwrap().take()
    }
}

impl HttpClientExt for CustomClient {
    fn send<T, U>(
        &self,
        req: rig::http_client::Request<T>,
    ) -> impl Future<Output = HttpResult<HttpResponse<LazyBody<U>>>> + Send + 'static
    where
        T: Into<Bytes> + Send,
        U: From<Bytes> + Send + 'static,
    {
        let (parts, body) = req.into_parts();
        let req = self
            .inner
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .body(body.into());

        let reasoning_handle = self.last_reasoning.clone();

        async move {
            let response = req
                .send()
                .await
                .map_err(|e| HttpError::Instance(Box::new(e)))?;
            if !response.status().is_success() {
                return Err(HttpError::InvalidStatusCodeWithMessage(
                    response.status(),
                    response.text().await.unwrap(),
                ));
            }

            let mut res = HttpResponse::builder().status(response.status());
            if let Some(hs) = res.headers_mut() {
                *hs = response.headers().clone();
            }

            let body: LazyBody<U> = Box::pin(async move {
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|e| HttpError::Instance(Box::new(e)))?;

                let mut json: serde_json::Value = serde_json::from_slice(&bytes)
                    .unwrap_or(serde_json::Value::Null);

                if let Some(choices) = json.get_mut("choices").and_then(|c| c.as_array_mut()) {
                    let mut reasoning_text: Option<String> = None;
                    for choice in choices.iter_mut() {
                        let msg = &mut choice["message"];
                        if msg["role"].as_str() == Some("") {
                            msg["role"] = serde_json::Value::String("assistant".into());
                        }
                        if let Some(obj) = msg.as_object_mut() {
                            if let Some(reasoning) = obj.remove("reasoning_content") {
                                if reasoning_text.is_none() {
                                    reasoning_text = reasoning.as_str().map(|s| s.to_string());
                                }
                            }
                        }
                    }
                    if let Some(text) = reasoning_text {
                        *reasoning_handle.lock().unwrap() = Some(text);
                    }
                }

                let fixed = serde_json::to_vec(&json)
                    .map_err(|e| HttpError::Instance(Box::new(e)))?;
                Ok(U::from(Bytes::from(fixed)))
            });

            res.body(body).map_err(HttpError::Protocol)
        }
    }

    fn send_multipart<U>(
        &self,
        req: rig::http_client::Request<MultipartForm>,
    ) -> impl Future<Output = HttpResult<HttpResponse<LazyBody<U>>>> + Send + 'static
    where
        U: From<Bytes> + Send + 'static,
    {
        <reqwest::Client as HttpClientExt>::send_multipart(&self.inner, req)
    }

    fn send_streaming<T>(
        &self,
        req: rig::http_client::Request<T>,
    ) -> impl Future<Output = HttpResult<StreamingResponse>> + Send
    where
        T: Into<Bytes>,
    {
        <reqwest::Client as HttpClientExt>::send_streaming(&self.inner, req)
    }
}

/// Build a rig completions client (Chat Completions API) with CustomClient.
///
/// Returns `(CompletionsClient, CustomClient)` so callers can extract reasoning.
pub fn build_completions_client(
    api_key: &str,
    api_base: Option<&str>,
    timeout_seconds: Option<u64>,
) -> Result<(openai::CompletionsClient<CustomClient>, CustomClient), String> {
    let mut reqwest_builder = reqwest::Client::builder();
    if let Some(secs) = timeout_seconds {
        reqwest_builder = reqwest_builder.timeout(std::time::Duration::from_secs(secs));
    }
    let reqwest_client = reqwest_builder
        .build()
        .map_err(|e| format!("reqwest client build: {e}"))?;

    let custom = CustomClient::new(reqwest_client);

    let mut builder = openai::Client::builder()
        .api_key(api_key)
        .http_client(custom.clone());

    if let Some(base) = api_base {
        builder = builder.base_url(base.trim_end_matches('/'));
    }

    let client = builder
        .build()
        .map_err(|e| format!("rig client build: {e}"))?
        .completions_api();

    Ok((client, custom))
}

/// Build a rig client for embedding requests.
///
/// Uses a plain openai::Client (no CustomClient needed — embeddings don't have role/reasoning issues).
pub fn build_embedding_client(
    api_key: &str,
    api_base: Option<&str>,
) -> Result<openai::Client, String> {
    let mut builder = openai::Client::builder().api_key(api_key);

    if let Some(base) = api_base {
        builder = builder.base_url(base.trim_end_matches('/'));
    }

    builder.build().map_err(|e| format!("rig embedding client build: {e}"))
}
