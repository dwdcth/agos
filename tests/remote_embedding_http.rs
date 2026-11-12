//! Remote-HTTP embedding backend tests against a local mock server.
//!
//! The mock speaks the OpenAI-compatible `POST /embeddings` dialect and
//! implements a tiny deterministic "semantic model" so ingest -> hybrid
//! search round-trips without network access.

use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::config::{
        Config, EmbeddingBackend, EmbeddingConfig, RetrievalConfig, RetrievalMode,
        RootEmbeddingRuntimeConfig, RootVectorConfig, VectorBackend,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::embedding::RemoteHttpEmbeddingGenerator,
    memory::record::{RecordType, Scope, TruthLayer},
    search::{QueryStrategy, SearchRequest, SearchService},
};
use serde_json::{Value, json};

/// One recorded inbound request.
#[derive(Debug, Clone)]
struct RecordedRequest {
    path: String,
    authorization: Option<String>,
    body: Value,
}

impl RecordedRequest {
    fn input_texts(&self) -> Vec<String> {
        self.body
            .get("input")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }
}

type Handler = Arc<dyn Fn(usize, &RecordedRequest) -> (u16, String) + Send + Sync>;

struct MockEmbeddingServer {
    base_url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    _shutdown: Sender<()>,
}

impl MockEmbeddingServer {
    fn start(handler: Handler) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock server");
        let addr = listener.local_addr().expect("mock server addr");
        let requests: Arc<Mutex<Vec<RecordedRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        let recorded = Arc::clone(&requests);
        thread::spawn(move || {
            let mut served = 0usize;
            for stream in listener.incoming() {
                if rx.try_recv().is_ok() {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let Some(request) = read_request(&mut stream) else {
                    continue;
                };
                let (status, body) = handler(served, &request);
                served += 1;
                recorded.lock().unwrap().push(request);
                let response = format!(
                    "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    if status == 200 { "OK" } else { "Error" },
                    body.len(),
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        Self {
            base_url: format!("http://{addr}/v1"),
            requests,
            _shutdown: tx,
        }
    }

    fn recorded(&self) -> Vec<RecordedRequest> {
        self.requests.lock().unwrap().clone()
    }
}

fn read_request(stream: &mut std::net::TcpStream) -> Option<RecordedRequest> {
    let mut buffer = [0u8; 8192];
    let mut raw = Vec::new();
    // read until end of headers, then content-length body
    loop {
        let n = stream.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        raw.extend_from_slice(&buffer[..n]);
        if let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&raw[..header_end]).to_string();
            let content_length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())?
                })
                .unwrap_or(0);
            if raw.len() >= header_end + 4 + content_length {
                let path = head
                    .lines()
                    .next()
                    .and_then(|request_line| request_line.split_whitespace().nth(1))
                    .unwrap_or_default()
                    .to_string();
                let authorization = head.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("authorization")
                        .then(|| value.trim().to_string())
                });
                let body = serde_json::from_slice::<Value>(
                    &raw[header_end + 4..header_end + 4 + content_length],
                )
                .unwrap_or(Value::Null);
                return Some(RecordedRequest {
                    path,
                    authorization,
                    body,
                });
            }
        }
    }
    None
}

/// The mock "model": feline/cat → e0, canine/dog → e1, else a third vector.
fn semantic_vector(text: &str) -> Vec<f32> {
    let lowered = text.to_lowercase();
    if lowered.contains("feline") || lowered.contains("cat") {
        vec![1.0, 0.0, 0.0, 0.0]
    } else if lowered.contains("canine") || lowered.contains("dog") {
        vec![0.0, 1.0, 0.0, 0.0]
    } else {
        vec![0.0, 0.0, 1.0, 0.0]
    }
}

fn embeddings_response(texts: &[String], dimensions: Option<usize>) -> String {
    let data = texts
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let mut embedding = semantic_vector(text);
            if let Some(dimensions) = dimensions {
                embedding = vec![0.01; dimensions];
            }
            json!({ "index": index, "embedding": embedding })
        })
        .collect::<Vec<_>>();
    json!({ "data": data, "model": "BAAI/bge-m3" }).to_string()
}

fn generator_config(base_url: &str) -> RootEmbeddingRuntimeConfig {
    RootEmbeddingRuntimeConfig {
        provider: "remote_http".to_string(),
        model: "BAAI/bge-m3".to_string(),
        api_base: Some(base_url.to_string()),
        api_key: Some("sk-mock".to_string()),
        dimensions: Some(4),
        batch_size: None,
        timeout_seconds: Some(10),
    }
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Runtime::new().expect("test runtime")
}

#[test]
fn remote_generator_splits_input_at_the_32_text_provider_cap() {
    let server = MockEmbeddingServer::start(Arc::new(|_, request| {
        (200, embeddings_response(&request.input_texts(), None))
    }));

    let generator = RemoteHttpEmbeddingGenerator::from_config(&generator_config(&server.base_url))
        .expect("generator should build");
    assert_eq!(generator.batch_size(), 32, "bge-m3 provider cap default");

    let texts = (0..65)
        .map(|i| format!("feline record number {i}"))
        .collect::<Vec<_>>();
    let embeddings = runtime()
        .block_on(generator.generate_embeddings(texts))
        .expect("65 texts should embed");

    assert_eq!(embeddings.len(), 65, "one embedding per input text");
    assert!(
        embeddings.iter().all(|embedding| embedding.len() == 4),
        "all vectors carry the configured dimensionality"
    );

    let requests = server.recorded();
    assert_eq!(requests.len(), 3, "65 texts at batch cap 32 -> 3 requests");
    assert_eq!(
        requests
            .iter()
            .map(|r| r.input_texts().len())
            .collect::<Vec<_>>(),
        vec![32, 32, 1],
        "batches respect the 32-text provider cap"
    );
    assert!(
        requests
            .iter()
            .all(|r| r.path == "/v1/embeddings" && r.body["model"] == "BAAI/bge-m3"),
        "requests target the OpenAI-compatible embeddings endpoint with the model"
    );
    assert!(
        requests
            .iter()
            .all(|r| r.authorization.as_deref() == Some("Bearer sk-mock")),
        "api key rides the Authorization header"
    );
}

#[test]
fn remote_generator_retries_transient_failures_with_backoff() {
    // Fail twice, then succeed: 1 initial attempt + 2 retries recovers.
    let server = MockEmbeddingServer::start(Arc::new(|attempt, request| {
        if attempt < 2 {
            (503, json!({"error": "upstream busy"}).to_string())
        } else {
            (200, embeddings_response(&request.input_texts(), None))
        }
    }));

    let generator = RemoteHttpEmbeddingGenerator::from_config(&generator_config(&server.base_url))
        .expect("generator should build");
    let embeddings = runtime()
        .block_on(generator.generate_embeddings(vec!["feline one".into()]))
        .expect("retry budget absorbs two transient 503s");
    assert_eq!(embeddings.len(), 1);
    assert_eq!(server.recorded().len(), 3, "exactly three attempts");
}

#[test]
fn remote_generator_gives_up_after_the_retry_budget() {
    let server =
        MockEmbeddingServer::start(Arc::new(|_, _| (500, json!({"error": "down"}).to_string())));

    let generator = RemoteHttpEmbeddingGenerator::from_config(&generator_config(&server.base_url))
        .expect("generator should build");
    let error = runtime()
        .block_on(generator.generate_embeddings(vec!["feline one".into()]))
        .expect_err("persistent failures must surface");
    assert!(
        error.to_string().contains("500"),
        "error carries status: {error}"
    );
    assert_eq!(
        server.recorded().len(),
        3,
        "1 initial attempt + 2 retries, then stop"
    );
}

#[test]
fn remote_generator_fails_fast_on_authentication_errors() {
    let server = MockEmbeddingServer::start(Arc::new(|_, _| {
        (401, json!({"error": "bad key"}).to_string())
    }));

    let generator = RemoteHttpEmbeddingGenerator::from_config(&generator_config(&server.base_url))
        .expect("generator should build");
    let error = runtime()
        .block_on(generator.generate_embeddings(vec!["feline one".into()]))
        .expect_err("401 must not be retried");
    assert!(error.to_string().contains("401"));
    assert_eq!(server.recorded().len(), 1, "no retries for auth failures");
}

#[test]
fn remote_generator_validates_dimension_contract() {
    // Server ignores semantics and returns 8-dim vectors while the config
    // declares 4: the mismatch must fail loudly instead of persisting junk.
    let server = MockEmbeddingServer::start(Arc::new(|_, request| {
        (200, embeddings_response(&request.input_texts(), Some(8)))
    }));

    let generator = RemoteHttpEmbeddingGenerator::from_config(&generator_config(&server.base_url))
        .expect("generator should build");
    let error = runtime()
        .block_on(generator.generate_embeddings(vec!["feline one".into()]))
        .expect_err("dimension mismatch must be rejected");
    assert!(
        error.to_string().contains("dimension mismatch"),
        "error names the violation: {error}"
    );
    assert!(
        error.to_string().contains("expected 4"),
        "error carries the expected dimensionality: {error}"
    );
}

#[test]
fn remote_generator_requires_endpoint_configuration() {
    let config = RootEmbeddingRuntimeConfig {
        provider: "remote_http".to_string(),
        model: "BAAI/bge-m3".to_string(),
        api_base: None,
        api_key: Some("sk-mock".to_string()),
        ..Default::default()
    };
    let error = match RemoteHttpEmbeddingGenerator::from_config(&config) {
        Ok(_) => panic!("missing api_base must fail at build time"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("api_base"));
}

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-remote-embedding-tests")
        .join(format!("{name}-{unique}"))
        .join("remote-embedding.sqlite")
}

fn remote_test_config(db_path: &std::path::Path, base_url: &str, mode: RetrievalMode) -> Config {
    Config {
        db_path: db_path.display().to_string(),
        retrieval: RetrievalConfig { mode },
        embedding: EmbeddingConfig {
            backend: EmbeddingBackend::RemoteHttp,
            model: Some("BAAI/bge-m3".to_string()),
            endpoint: Some(base_url.to_string()),
            api_key: Some("sk-mock".to_string()),
            dimensions: Some(4),
            batch_size: Some(32),
            timeout_seconds: Some(10),
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            table: "object_embeddings_vec".to_string(),
            similarity: "cosine".to_string(),
        },
        ..Default::default()
    }
}

fn ingest(service: &IngestService<'_>, uri: &str, content: &str) {
    service
        .ingest(IngestRequest {
            source_uri: uri.to_string(),
            source_label: Some(uri.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T10:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
}

#[test]
fn remote_http_backend_end_to_end_ingest_to_hybrid_semantic_recall() {
    let server = MockEmbeddingServer::start(Arc::new(|_, request| {
        (200, embeddings_response(&request.input_texts(), None))
    }));
    let db_path = fresh_db_path("e2e-hybrid");
    let config = remote_test_config(&db_path, &server.base_url, RetrievalMode::Hybrid);

    let db = Database::open(&db_path).expect("database should open");
    let ingest_service = IngestService::with_config(db.conn(), &config);
    ingest(
        &ingest_service,
        "memo://project/feline",
        "feline companion naps all afternoon",
    );
    ingest(
        &ingest_service,
        "memo://project/canine",
        "canine buddy runs across the yard",
    );

    // The vectors must be mirrored into the sqlite-vec index on write.
    let vec_rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM object_embeddings_vec", [], |row| {
            row.get(0)
        })
        .expect("vec0 table should exist after ingest");
    assert_eq!(vec_rows, 2, "ingest dual-writes every chunk vector");

    // "cat friend" shares no lexical token with the feline doc; only the
    // embedding channel can recall it.
    let response = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("cat friend"))
        .expect("hybrid search should succeed");

    assert!(
        response
            .results
            .iter()
            .any(|result| result.record.source.uri == "memo://project/feline"
                && result
                    .trace
                    .query_strategies
                    .contains(&QueryStrategy::Embedding)),
        "semantic neighbor should be recalled through the embedding channel: {:?}",
        response
            .results
            .iter()
            .map(|result| result.record.source.uri.clone())
            .collect::<Vec<_>>()
    );
    assert!(
        response
            .results
            .iter()
            .all(|result| result.record.source.uri != "memo://project/canine"
                || !result
                    .trace
                    .query_strategies
                    .contains(&QueryStrategy::Embedding)),
        "the canine doc must not outrank the feline doc semantically"
    );

    let requests = server.recorded();
    assert!(
        requests.len() >= 3,
        "two ingest batches + one query embedding: {}",
        requests.len()
    );
}
