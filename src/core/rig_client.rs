use bytes::Bytes;
use rig_core::completion::{AssistantContent, CompletionRequest};
use rig_core::http_client::{
    Error as HttpError, HttpClientExt, LazyBody, MultipartForm, Response as HttpResponse,
    Result as HttpResult, StreamingResponse,
};
use rig_core::providers::openai::{self, OpenAI};
use rig_core::wasm_compat::WasmCompatSend;
use rig_reqwest::ReqwestClient;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

/// Custom HTTP client for OpenAI-compatible APIs.
///
/// Handles provider quirks at the HTTP response level:
/// - Fixes empty `role` (some APIs return `""` instead of `"assistant"`)
/// - Extracts `reasoning_content` (Qwen/DeepSeek thinking mode)
///
/// `send` carries the interception logic; multipart and streaming requests
/// (unused by agent-memos) delegate to the stock reqwest transport.
#[derive(Clone, Debug, Default)]
pub struct CustomClient {
    inner: reqwest::Client,
    transport: Option<ReqwestClient>,
    last_reasoning: Arc<Mutex<Option<String>>>,
}

impl CustomClient {
    pub fn new(inner: reqwest::Client) -> Self {
        Self {
            transport: Some(ReqwestClient::from(inner.clone())),
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
        req: rig_core::http_client::Request<T>,
    ) -> impl Future<Output = HttpResult<HttpResponse<LazyBody<U>>>> + WasmCompatSend + 'static
    where
        T: Into<Bytes> + WasmCompatSend,
        U: From<Bytes> + WasmCompatSend + 'static,
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
                let status = response.status();
                let headers = response.headers().clone();
                let body = response.text().await.unwrap_or_default();
                return Err(HttpError::non_success_with_details(status, headers, body));
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

                let mut json: serde_json::Value =
                    serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);

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

                let fixed =
                    serde_json::to_vec(&json).map_err(|e| HttpError::Instance(Box::new(e)))?;
                Ok(U::from(Bytes::from(fixed)))
            });

            res.body(body).map_err(HttpError::Protocol)
        }
    }

    fn send_multipart<U>(
        &self,
        req: rig_core::http_client::Request<MultipartForm>,
    ) -> impl Future<Output = HttpResult<HttpResponse<LazyBody<U>>>> + WasmCompatSend + 'static
    where
        U: From<Bytes> + WasmCompatSend + 'static,
    {
        // multipart 无需 role/reasoning 拦截，直接走标准 reqwest 传输
        <ReqwestClient as HttpClientExt>::send_multipart(self.transport.as_ref().unwrap(), req)
    }

    fn send_streaming<T>(
        &self,
        req: rig_core::http_client::Request<T>,
    ) -> impl Future<Output = HttpResult<StreamingResponse>> + WasmCompatSend
    where
        T: Into<Bytes> + WasmCompatSend,
    {
        // 流式同样无需拦截，走标准 reqwest 传输
        <ReqwestClient as HttpClientExt>::send_streaming(self.transport.as_ref().unwrap(), req)
    }
}

/// One structured completion: prompt + preamble + sampling parameters.
#[derive(Debug, Clone)]
pub struct StructuredCompletion {
    pub model: String,
    pub preamble: String,
    pub prompt: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
    /// Provider-specific passthrough（如 Qwen 的 enable_thinking）
    pub additional_params: Option<serde_json::Value>,
}

/// A completions endpoint: provider client + the custom transport that
/// captures `reasoning_content` and fixes empty roles.
#[derive(Clone, Debug)]
pub struct CompletionsHandle {
    client: OpenAI,
    custom: CustomClient,
}

impl CompletionsHandle {
    /// Run a structured (JSON-schema-constrained) completion and parse the
    /// reply into `T`. The schema is sent via `output_schema` when the
    /// provider supports it; the reply text is parsed leniently regardless.
    pub async fn complete_json<T>(&self, params: StructuredCompletion) -> Result<T, String>
    where
        T: DeserializeOwned + JsonSchema,
    {
        let model = self.client.chat(&params.model);

        let mut request =
            CompletionRequest::new(params.prompt.clone()).preamble(params.preamble.clone());
        request.temperature = params.temperature;
        request.max_tokens = params.max_tokens;
        request.additional_params = params.additional_params;
        request.output_schema = Some(schemars::schema_for!(T));

        let response = model
            .call(request)
            .await
            .map_err(|e| format!("rig completion request failed: {e}"))?;

        let text = response
            .choice
            .iter()
            .filter_map(|content| match content {
                AssistantContent::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("");

        parse_json_output(&text)
    }

    /// Take the reasoning_content captured by the custom transport.
    pub fn take_reasoning(&self) -> Option<String> {
        self.custom.take_reasoning()
    }
}

/// Parse a model reply into `T`, tolerating markdown code fences.
fn parse_json_output<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    let trimmed = text.trim();
    if let Ok(value) = serde_json::from_str(trimmed) {
        return Ok(value);
    }
    // 剥离 ```json ... ``` 围栏后再试
    let stripped = trimmed
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if let Ok(value) = serde_json::from_str(stripped) {
        return Ok(value);
    }
    // 取第一个能配平且可解析的 JSON 对象再试：思维链模型的 content 常带
    // 残渣（spark 思考模式泄漏 </think> 与原生 <tool_call> 伪 XML，Qwen 系
    // 泄漏 }{ 片段）——首尾垃圾不应连累完全合法的主体 JSON。逐个 `{`
    // 起点尝试（孤立泄漏花括号不是合法起点）。
    for (start, b) in stripped.bytes().enumerate() {
        if b != b'{' {
            continue;
        }
        if let Some(end) = balanced_end(stripped, start) {
            if let Ok(value) = serde_json::from_str::<T>(&stripped[start..=end]) {
                return Ok(value);
            }
        }
    }
    let snippet: String = trimmed.chars().take(200).collect();
    Err(format!(
        "failed to parse structured completion output: output head: {snippet}"
    ))
}

/// 从 `start`（`{` 的字节下标）出发找配平的结束 `}` 下标（字符串感知：
/// 跳过字符串字面量内的花括号；不递归修复畸形——那是模型的事）。
fn balanced_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'{') {
        return None;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Build a rig completions endpoint (Chat Completions route) over the
/// CustomClient transport. Use [`CompletionsHandle::complete_json`] for
/// structured requests; `take_reasoning` extracts Qwen/DeepSeek thinking.
pub fn build_completions_client(
    api_key: &str,
    api_base: Option<&str>,
    timeout_seconds: Option<u64>,
) -> Result<CompletionsHandle, String> {
    let mut reqwest_builder = reqwest::Client::builder();
    if let Some(secs) = timeout_seconds {
        reqwest_builder = reqwest_builder.timeout(std::time::Duration::from_secs(secs));
    }
    let reqwest_client = reqwest_builder
        .build()
        .map_err(|e| format!("reqwest client build: {e}"))?;

    let custom = CustomClient::new(reqwest_client);

    let mut config = openai::OpenAIConfig::new(api_key);
    if let Some(base) = api_base {
        config = config.with_base_url(base.trim_end_matches('/'));
    }
    let client = config.connect(custom.clone());

    Ok(CompletionsHandle { client, custom })
}

/// Build a rig client for embedding requests.
///
/// Uses a plain `reqwest` transport (no CustomClient needed — embeddings
/// don't have role/reasoning issues), default route.
pub fn build_embedding_client(api_key: &str, api_base: Option<&str>) -> Result<OpenAI, String> {
    let custom = CustomClient::new(reqwest::Client::new());
    let mut config = openai::OpenAIConfig::new(api_key);
    if let Some(base) = api_base {
        config = config.with_base_url(base.trim_end_matches('/'));
    }
    Ok(config.connect(custom))
}

#[cfg(test)]
mod parse_tests {
    use super::parse_json_output;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct Decision {
        action: String,
    }

    /// 思维链残渣不应连累合法 JSON（spark 思考模式实测泄漏
    /// </think> 与原生 <tool_call> 伪 XML）。
    #[test]
    fn tolerates_thinking_residue() {
        let out = parse_json_output::<Decision>(
            r#"{"action":"tool"}</think><tool_call>memory_search</tool_call>"#,
        )
        .expect("should parse");
        assert_eq!(out.action, "tool");
    }

    /// 前后散文（含围栏外的说明文字与泄漏的 }{ 片段）。
    #[test]
    fn tolerates_surrounding_prose() {
        let out = parse_json_output::<Decision>(
            "好的，我的判断如下：}{\n{\"action\":\"reply\"}\n以上就是全部。",
        )
        .expect("should parse");
        assert_eq!(out.action, "reply");
    }

    /// 字符串字面量内的花括号不参与配平。
    #[test]
    fn braces_inside_strings_ignored() {
        let out = parse_json_output::<Decision>(r#"前缀 {"action":"a}b{c"} 后缀"#)
            .expect("should parse");
        assert_eq!(out.action, "a}b{c");
    }
}
