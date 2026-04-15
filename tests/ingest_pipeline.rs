use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    ingest::{
        IngestRequest, IngestService,
        chunk::{ChunkConfig, chunk_source},
        detect::{Format, detect_format},
        normalize::{NormalizedSource, normalize_source},
    },
    memory::{
        record::{ChunkAnchor, RecordType, Scope, SourceKind, TruthLayer},
        repository::MemoryRepository,
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-tests")
        .join(format!("{name}-{unique}"))
        .join("nested")
        .join("ingest.sqlite")
}

fn plain_note_request() -> IngestRequest {
    IngestRequest {
        source_uri: "file:///tmp/daily-note.md".to_string(),
        source_label: Some("daily-note".to_string()),
        source_kind: Some(SourceKind::Note),
        content: "# Daily Log\n\n- capture retrieval tasks\n- preserve citations".to_string(),
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        recorded_at: "2026-04-15T10:00:00Z".to_string(),
        valid_from: None,
        valid_to: None,
    }
}

fn conversation_request() -> IngestRequest {
    IngestRequest {
        source_uri: "file:///tmp/agent-chat.json".to_string(),
        source_label: Some("agent-chat".to_string()),
        source_kind: None,
        content: r#"
[
  {"role":"user","content":"Summarize the migration plan."},
  {"role":"assistant","content":"I will inspect the authority store."},
  {"role":"user","content":"Keep the retrieval modes unchanged."},
  {"role":"assistant","content":"I will preserve lexical_only, embedding_only, and hybrid."}
]
"#
        .to_string(),
        scope: Scope::Session,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T3,
        recorded_at: "2026-04-15T11:00:00Z".to_string(),
        valid_from: Some("2026-04-15T11:00:00Z".to_string()),
        valid_to: Some("2026-04-16T11:00:00Z".to_string()),
    }
}

#[test]
fn ingest_normalizes_supported_sources() {
    let note_request = plain_note_request();
    let note_format = detect_format(&note_request.content);
    assert_eq!(note_format, Format::PlainText);

    let normalized_note =
        normalize_source(&note_request, note_format).expect("plain note should normalize");
    assert_eq!(normalized_note.source_kind, SourceKind::Note);
    assert_eq!(normalized_note.canonical_uri, note_request.source_uri);
    assert_eq!(
        normalized_note.text,
        "# Daily Log\n\n- capture retrieval tasks\n- preserve citations"
    );

    let chat_request = conversation_request();
    let chat_format = detect_format(&chat_request.content);
    assert_eq!(chat_format, Format::ChatGptJson);

    let normalized_chat =
        normalize_source(&chat_request, chat_format).expect("conversation export should normalize");
    assert_eq!(normalized_chat.source_kind, SourceKind::Conversation);
    assert_eq!(
        normalized_chat.text,
        "> Summarize the migration plan.\nI will inspect the authority store.\n> Keep the retrieval modes unchanged.\nI will preserve lexical_only, embedding_only, and hybrid."
    );
}

#[test]
fn chunking_preserves_order_and_line_or_turn_anchors() {
    let note =
        normalize_source(&plain_note_request(), Format::PlainText).expect("note should normalize");
    let text_chunks = chunk_source(
        &note,
        ChunkConfig {
            text_char_window: 32,
            text_char_overlap: 0,
            conversation_turn_window: 1,
        },
    );
    assert!(
        text_chunks.len() >= 2,
        "small chunk window should split note into multiple chunks"
    );
    assert_eq!(text_chunks[0].chunk_index, 0);
    assert_eq!(text_chunks[0].chunk_count, text_chunks.len() as u32);
    assert!(matches!(
        text_chunks[0].anchor,
        ChunkAnchor::LineRange { .. } | ChunkAnchor::CharRange { .. }
    ));

    let chat = normalize_source(&conversation_request(), Format::ChatGptJson)
        .expect("chat should normalize");
    let conversation_chunks = chunk_source(
        &chat,
        ChunkConfig {
            text_char_window: 64,
            text_char_overlap: 0,
            conversation_turn_window: 1,
        },
    );
    assert_eq!(conversation_chunks.len(), 2);
    assert!(matches!(
        conversation_chunks[0].anchor,
        ChunkAnchor::TurnRange {
            start_turn: 1,
            end_turn: 2
        }
    ));
    assert!(matches!(
        conversation_chunks[1].anchor,
        ChunkAnchor::TurnRange {
            start_turn: 3,
            end_turn: 4
        }
    ));
}

#[test]
fn ingest_persists_chunk_provenance_and_validity_metadata() {
    let path = fresh_db_path("pipeline");
    let db = Database::open(&path).expect("database should bootstrap");
    let service = IngestService::with_chunk_config(
        db.conn(),
        ChunkConfig {
            text_char_window: 64,
            text_char_overlap: 0,
            conversation_turn_window: 1,
        },
    );

    let request = conversation_request();
    let report = service.ingest(request).expect("ingest should succeed");

    assert_eq!(report.detected_format, Format::ChatGptJson);
    assert_eq!(report.chunk_count, 2);
    assert_eq!(report.record_ids.len(), 2);
    assert_eq!(
        report.normalized_source.source_kind,
        SourceKind::Conversation
    );

    let repo = MemoryRepository::new(db.conn());
    let stored = repo.list_records().expect("records should load");
    assert_eq!(stored.len(), 2);
    assert_eq!(stored[0].source.uri, "file:///tmp/agent-chat.json");
    assert_eq!(stored[0].source.label.as_deref(), Some("agent-chat"));
    assert_eq!(
        stored[0].validity.valid_from.as_deref(),
        Some("2026-04-15T11:00:00Z")
    );
    assert_eq!(
        stored[0].validity.valid_to.as_deref(),
        Some("2026-04-16T11:00:00Z")
    );
    assert_eq!(stored[0].provenance.origin, "ingest");
    assert_eq!(
        stored[0].provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(stored[0].provenance.derived_from[0].contains("#turn-1-2"));
    assert_eq!(
        stored[0]
            .chunk
            .as_ref()
            .expect("chunk metadata")
            .chunk_index,
        0
    );
    assert_eq!(
        stored[1]
            .chunk
            .as_ref()
            .expect("chunk metadata")
            .chunk_index,
        1
    );
    assert_eq!(
        stored[0]
            .chunk
            .as_ref()
            .expect("chunk metadata")
            .chunk_count,
        2
    );
    assert!(matches!(
        stored[0].chunk.as_ref().expect("chunk metadata").anchor,
        ChunkAnchor::TurnRange {
            start_turn: 1,
            end_turn: 2
        }
    ));
    assert!(
        !stored[0]
            .chunk
            .as_ref()
            .expect("chunk metadata")
            .content_hash
            .is_empty()
    );
    assert_eq!(stored[0].id, report.record_ids[0]);
    assert_eq!(stored[1].id, report.record_ids[1]);
}
