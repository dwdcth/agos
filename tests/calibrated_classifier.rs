//! 可校准统计分类层集成测试：库内 DSL sidecar 训练、关键词优先级、
//! ingest 门控（默认关）与留出评估。

use std::path::PathBuf;

use agent_memos::{
    core::config::{Config, MemoryConfig},
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        calibrated::{
            CalibratedLayer, CalibratedTaxonomyClassifier, CalibratedTrainingSample,
            evaluate_calibrated_classifier,
        },
        classifier::ClassificationInput,
        dsl::FlatFactDslRecordV1,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        store::{FactDslStore, PersistedFactDslRecordV1},
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-calibrated-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

fn labeled_record(id: &str, uri: &str, content: &str) -> MemoryRecord {
    let now = "2026-11-12T10:00:00Z";
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: uri.to_string(),
            kind: SourceKind::Note,
            label: None,
        },
        timestamp: RecordTimestamp {
            recorded_at: now.to_string(),
            created_at: now.to_string(),
            updated_at: now.to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: content.to_string(),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 3,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow {
            valid_from: Some(now.to_string()),
            valid_to: None,
        },
    }
}

fn labeled_dsl(record: &MemoryRecord, domain: &str, topic: &str) -> PersistedFactDslRecordV1 {
    PersistedFactDslRecordV1 {
        record_id: record.id.clone(),
        payload: FlatFactDslRecordV1 {
            domain: domain.to_string(),
            topic: topic.to_string(),
            aspect: "behavior".to_string(),
            kind: "fact".to_string(),
            claim: record.content_text.clone(),
            truth_layer: "t2".to_string(),
            source_ref: record.source.uri.clone(),
            why: None,
            time: None,
            cond: None,
            impact: None,
            conf: Some(0.9),
            rel: None,
        },
        classification_confidence: Some(0.9),
        needs_review: false,
    }
}

/// 8 条两类可分样本：k8s 词表 → external/provider；检索词表 → project/retrieval。
fn seed_labeled_sidecars(conn: &rusqlite::Connection) {
    let repository = MemoryRepository::new(conn);
    for i in 0..4 {
        let k8s = labeled_record(
            &format!("k8s-{i}"),
            &format!("memo://k8s/{i}"),
            &format!("fleet upgrade window pager duty incident quota {i}"),
        );
        let retrieval = labeled_record(
            &format!("retr-{i}"),
            &format!("memo://retr/{i}"),
            &format!("lexical baseline sqlite fts5 hybrid merge scan {i}"),
        );
        repository.insert_record(&k8s).expect("insert k8s");
        repository
            .insert_record(&retrieval)
            .expect("insert retrieval");
        repository
            .put_fact_dsl(&labeled_dsl(&k8s, "external", "provider"))
            .expect("dsl k8s");
        repository
            .put_fact_dsl(&labeled_dsl(&retrieval, "project", "retrieval"))
            .expect("dsl retrieval");
    }
}

#[test]
fn fit_from_repository_and_statistical_fallback() {
    let path = fresh_db_path("fit");
    std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create temp dir");
    let db = Database::open(&path).expect("open db");
    seed_labeled_sidecars(db.conn());

    let repository = MemoryRepository::new(db.conn());
    let classifier = CalibratedTaxonomyClassifier::fit_from_repository(&repository)
        .expect("fit from repository");
    assert!(
        classifier.statistical_available(),
        "8 条标注样本超过训练门槛"
    );

    // 无关键词命中的新文本 → 统计层兜底到 external
    let (output, report) = classifier
        .classify_with_report_sync(&ClassificationInput::new(
            "memo://calibrate/probe",
            "fleet upgrade window pager duty escalation",
        ))
        .expect("classify probe");
    assert_eq!(report.layer, CalibratedLayer::Statistical);
    assert_eq!(output.taxonomy.domain.as_str(), "external");
    assert_eq!(output.taxonomy.topic.as_str(), "provider");

    // 关键词规则命中 → 规则优先，统计层不越权
    let (rule_output, rule_report) = classifier
        .classify_with_report_sync(&ClassificationInput::new(
            "roadmap#phase9",
            "Use lexical-first retrieval as the baseline decision because explainability matters.",
        ))
        .expect("classify rule-hit");
    assert_eq!(rule_report.layer, CalibratedLayer::Keyword);
    assert_eq!(rule_output.taxonomy.topic.as_str(), "retrieval");

    let _ = std::fs::remove_dir_all(path.parent().expect("parent dir"));
}

#[test]
fn ingest_gate_off_by_default_and_on_when_configured() {
    let path_off = fresh_db_path("ingest-off");
    std::fs::create_dir_all(path_off.parent().expect("parent dir")).expect("create temp dir");
    let db_off = Database::open(&path_off).expect("open db");
    seed_labeled_sidecars(db_off.conn());

    // 默认配置：统计层不参与，规则未命中 → 默认 domain 回 observation/misc
    let default_config = Config::default();
    assert!(!default_config.memory.calibrated_classifier);
    let report = IngestService::with_config(db_off.conn(), &default_config)
        .ingest(IngestRequest {
            source_uri: "memo://probe/default".to_string(),
            source_label: None,
            source_kind: None,
            content: "fleet upgrade window pager duty escalation".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest default");
    let repository = MemoryRepository::new(db_off.conn());
    let dsl = repository
        .get_layered_record(&report.record_ids[0])
        .expect("layered record")
        .expect("record exists")
        .dsl
        .expect("dsl exists");
    assert_eq!(
        dsl.payload.domain, "project",
        "默认路径关键词 miss 回默认域（统计层未启用）"
    );

    let _ = std::fs::remove_dir_all(path_off.parent().expect("parent dir"));

    // 开启统计层：同文本分到 external
    let path_on = fresh_db_path("ingest-on");
    std::fs::create_dir_all(path_on.parent().expect("parent dir")).expect("create temp dir");
    let db_on = Database::open(&path_on).expect("open db");
    seed_labeled_sidecars(db_on.conn());

    let calibrated_config = Config {
        memory: MemoryConfig {
            calibrated_classifier: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let report = IngestService::with_config(db_on.conn(), &calibrated_config)
        .ingest(IngestRequest {
            source_uri: "memo://probe/calibrated".to_string(),
            source_label: None,
            source_kind: None,
            content: "fleet upgrade window pager duty escalation".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest calibrated");
    let repository = MemoryRepository::new(db_on.conn());
    let dsl = repository
        .get_layered_record(&report.record_ids[0])
        .expect("layered record")
        .expect("record exists")
        .dsl
        .expect("dsl exists");
    assert_eq!(
        dsl.payload.domain, "external",
        "统计层把未见词表分到训练时的域"
    );

    let _ = std::fs::remove_dir_all(path_on.parent().expect("parent dir"));
}

#[test]
fn holdout_evaluation_reports_accuracy_on_separable_data() {
    let mut samples = Vec::new();
    for i in 0..6 {
        samples.push(CalibratedTrainingSample {
            source_ref: format!("memo://k8s/{i}"),
            text: format!("fleet upgrade window pager duty incident quota {i}"),
            domain: agent_memos::memory::taxonomy::DomainV1::External,
            topic: agent_memos::memory::taxonomy::TopicV1::Provider,
            aspect: agent_memos::memory::taxonomy::AspectV1::Behavior,
            kind: agent_memos::memory::taxonomy::KindV1::Fact,
        });
        samples.push(CalibratedTrainingSample {
            source_ref: format!("memo://retr/{i}"),
            text: format!("lexical baseline sqlite fts5 hybrid merge scan {i}"),
            domain: agent_memos::memory::taxonomy::DomainV1::Project,
            topic: agent_memos::memory::taxonomy::TopicV1::Retrieval,
            aspect: agent_memos::memory::taxonomy::AspectV1::Behavior,
            kind: agent_memos::memory::taxonomy::KindV1::Fact,
        });
    }

    let report = evaluate_calibrated_classifier(&samples, 0.25);
    assert_eq!(report.total_records, 12);
    assert!(report.test_records > 0, "留出集非空");
    assert!(
        report.domain_accuracy >= 0.9,
        "两类高度可分样本的 domain 准确率应接近 1（实际 {}）",
        report.domain_accuracy
    );
}
