use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::{
    ingest::detect::{Format, extract_content_text, extract_message_text},
    memory::record::{RecordType, Scope, SourceKind, TruthLayer},
};

use super::IngestRequest;

pub type Result<T> = std::result::Result<T, NormalizeError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NormalizedSource {
    pub canonical_uri: String,
    pub source_label: Option<String>,
    pub source_kind: SourceKind,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub recorded_at: String,
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
    pub text: String,
    pub conversation_turns: Option<Vec<ConversationTurn>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConversationTurn {
    pub text: String,
    pub start_turn: u32,
    pub end_turn: u32,
}

#[derive(Debug, Error)]
pub enum NormalizeError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unsupported ChatGPT JSON shape")]
    UnsupportedChatGptShape,
    #[error("unsupported structured input for source {source_uri}")]
    UnsupportedStructuredInput { source_uri: String },
    #[error("missing required ingest field: {field}")]
    MissingField { field: &'static str },
}

pub fn normalize_source(request: &IngestRequest, format: Format) -> Result<NormalizedSource> {
    validate_request(request)?;

    let mut normalized = match format {
        Format::PlainText => normalize_plain_text(request),
        Format::ClaudeJsonl => normalize_claude_jsonl(request)?,
        Format::ChatGptJson => normalize_chatgpt_json(request)?,
        Format::CodexJsonl => normalize_codex_jsonl(request)?,
        Format::SlackJson => normalize_slack_json(request)?,
    };

    if normalized.text.trim().is_empty() && format != Format::PlainText {
        return Err(NormalizeError::UnsupportedStructuredInput {
            source_uri: request.source_uri.clone(),
        });
    }

    if let Some(source_kind) = request.source_kind {
        normalized.source_kind = source_kind;
    }

    Ok(normalized)
}

fn validate_request(request: &IngestRequest) -> Result<()> {
    if request.source_uri.trim().is_empty() {
        return Err(NormalizeError::MissingField {
            field: "source_uri",
        });
    }
    if request.recorded_at.trim().is_empty() {
        return Err(NormalizeError::MissingField {
            field: "recorded_at",
        });
    }

    Ok(())
}

fn normalize_plain_text(request: &IngestRequest) -> NormalizedSource {
    NormalizedSource {
        canonical_uri: request.source_uri.clone(),
        source_label: request.source_label.clone(),
        source_kind: request
            .source_kind
            .unwrap_or_else(|| infer_text_source_kind(&request.source_uri)),
        scope: request.scope,
        record_type: request.record_type,
        truth_layer: request.truth_layer,
        recorded_at: request.recorded_at.clone(),
        valid_from: request.valid_from.clone(),
        valid_to: request.valid_to.clone(),
        text: request.content.trim().to_string(),
        conversation_turns: None,
    }
}

fn normalize_claude_jsonl(request: &IngestRequest) -> Result<NormalizedSource> {
    let mut messages = Vec::new();

    for raw_line in request
        .content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let value: Value = serde_json::from_str(raw_line)?;
        let role = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("assistant")
            .to_string();
        let message = extract_message_text(&value).unwrap_or_default();
        if !message.trim().is_empty() {
            messages.push((role, message));
        }
    }

    Ok(render_conversation_source(request, messages))
}

fn normalize_chatgpt_json(request: &IngestRequest) -> Result<NormalizedSource> {
    let value: Value = serde_json::from_str(&request.content)?;

    let items = if let Some(messages) = value.as_array() {
        chatgpt_messages(messages)
    } else if let Some(messages) = value.get("messages").and_then(Value::as_array) {
        chatgpt_messages(messages)
    } else if let Some(mapping) = value.get("mapping").and_then(Value::as_object) {
        let mut ordered = Vec::new();
        if let Some(root_id) = find_root_node(mapping) {
            collect_messages_dfs(mapping, &root_id, &mut ordered);
        }
        ordered
    } else {
        return Err(NormalizeError::UnsupportedChatGptShape);
    };

    Ok(render_conversation_source(request, items))
}

fn normalize_codex_jsonl(request: &IngestRequest) -> Result<NormalizedSource> {
    let mut messages = Vec::new();

    for line in request
        .content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let value: Value = serde_json::from_str(line)?;
        if value.get("type").and_then(Value::as_str) != Some("event_msg") {
            continue;
        }
        let Some(payload) = value.get("payload") else {
            continue;
        };
        let message_type = payload.get("type").and_then(Value::as_str).unwrap_or("");
        let message = payload
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if message.is_empty() {
            continue;
        }

        match message_type {
            "user_message" => messages.push(("user".to_string(), message.to_string())),
            "agent_message" => messages.push(("assistant".to_string(), message.to_string())),
            _ => {}
        }
    }

    Ok(render_conversation_source(request, messages))
}

fn normalize_slack_json(request: &IngestRequest) -> Result<NormalizedSource> {
    let value: Value = serde_json::from_str(&request.content)?;
    let messages = value
        .as_array()
        .ok_or_else(|| NormalizeError::UnsupportedStructuredInput {
            source_uri: request.source_uri.clone(),
        })?;

    let mut speakers = Vec::new();
    let mut items = Vec::new();

    for message in messages {
        if message.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let speaker = message
            .get("user")
            .or_else(|| message.get("username"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let text = message
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if text.is_empty() {
            continue;
        }

        if !speakers.contains(&speaker) {
            speakers.push(speaker.clone());
        }
        let role = if speakers.first() == Some(&speaker) {
            "user"
        } else {
            "assistant"
        };
        items.push((role.to_string(), text.to_string()));
    }

    Ok(render_conversation_source(request, items))
}

fn render_conversation_source(
    request: &IngestRequest,
    items: Vec<(String, String)>,
) -> NormalizedSource {
    let mut lines = Vec::new();
    let mut turns = Vec::new();
    let mut current_lines = Vec::new();
    let mut current_start = 0u32;
    let mut last_turn = 0u32;

    for (index, (role, content)) in items.into_iter().enumerate() {
        let turn_number = index as u32 + 1;
        let rendered = if matches!(role.as_str(), "user" | "human") {
            format!("> {}", content.trim())
        } else {
            content.trim().to_string()
        };
        if rendered.trim().is_empty() {
            continue;
        }

        if rendered.starts_with("> ") && !current_lines.is_empty() {
            turns.push(ConversationTurn {
                text: current_lines.join("\n"),
                start_turn: current_start,
                end_turn: last_turn,
            });
            current_lines.clear();
        }

        if current_lines.is_empty() {
            current_start = turn_number;
        }
        last_turn = turn_number;
        lines.push(rendered.clone());
        current_lines.push(rendered);
    }

    if !current_lines.is_empty() {
        turns.push(ConversationTurn {
            text: current_lines.join("\n"),
            start_turn: current_start,
            end_turn: last_turn,
        });
    }

    NormalizedSource {
        canonical_uri: request.source_uri.clone(),
        source_label: request.source_label.clone(),
        source_kind: SourceKind::Conversation,
        scope: request.scope,
        record_type: request.record_type,
        truth_layer: request.truth_layer,
        recorded_at: request.recorded_at.clone(),
        valid_from: request.valid_from.clone(),
        valid_to: request.valid_to.clone(),
        text: lines.join("\n"),
        conversation_turns: Some(turns),
    }
}

fn infer_text_source_kind(source_uri: &str) -> SourceKind {
    if source_uri.ends_with(".md") || source_uri.ends_with(".markdown") {
        SourceKind::Note
    } else {
        SourceKind::Document
    }
}

fn chatgpt_messages(messages: &[Value]) -> Vec<(String, String)> {
    messages
        .iter()
        .filter_map(|message| {
            let role = message.get("role").and_then(Value::as_str)?;
            let content = message.get("content").and_then(extract_content_text)?;
            Some((role.to_string(), content))
        })
        .collect()
}

fn find_root_node(mapping: &serde_json::Map<String, Value>) -> Option<String> {
    mapping
        .iter()
        .find(|(_, node)| {
            node.get("parent")
                .is_none_or(|parent| parent.is_null() || parent.as_str() == Some(""))
        })
        .map(|(id, _)| id.clone())
}

fn collect_messages_dfs(
    mapping: &serde_json::Map<String, Value>,
    node_id: &str,
    result: &mut Vec<(String, String)>,
) {
    let Some(node) = mapping.get(node_id) else {
        return;
    };

    if let Some(message) = node.get("message") {
        let role = message
            .get("author")
            .and_then(|author| author.get("role"))
            .and_then(Value::as_str);
        let content = message.get("content").and_then(extract_content_text);

        if let (Some(role), Some(content)) = (role, content) {
            result.push((role.to_string(), content));
        }
    }

    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            if let Some(child_id) = child.as_str() {
                collect_messages_dfs(mapping, child_id, result);
            }
        }
    }
}
