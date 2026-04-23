use agent_memos::memory::classifier::{
    ClassificationInput, ClassificationOutput, KeywordTaxonomyClassifier, TaxonomyClassifier,
};
use agent_memos::memory::taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1};

#[test]
fn public_classifier_api_accepts_valid_output() {
    let input = ClassificationInput::new("roadmap#phase9", "Use lexical-first as baseline.");
    input
        .validate()
        .expect("classification input should validate");

    let output = ClassificationOutput::new(
        TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        0.82,
        false,
    )
    .expect("classification output should validate");

    assert_eq!(output.taxonomy.kind.as_str(), "decision");
    assert!(!output.needs_review);
}

#[test]
fn public_classifier_api_provides_observation_fallback() {
    let output = ClassificationOutput::fallback_observation(DomainV1::System);

    assert_eq!(output.taxonomy.domain.as_str(), "system");
    assert_eq!(output.taxonomy.topic.as_str(), "general");
    assert_eq!(output.taxonomy.kind.as_str(), "observation");
    assert!(output.needs_review);
}

#[tokio::test]
async fn public_keyword_classifier_produces_taxonomy_output() {
    let classifier = KeywordTaxonomyClassifier::default();
    let output = classifier
        .classify(&ClassificationInput::new(
            "roadmap#phase9",
            "Use lexical-first retrieval as the baseline decision because explainability matters.",
        ))
        .await
        .expect("keyword classifier should classify");

    assert_eq!(output.taxonomy.domain, DomainV1::Project);
    assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
    assert_eq!(output.taxonomy.aspect, AspectV1::Behavior);
    assert_eq!(output.taxonomy.kind, KindV1::Decision);
}

#[tokio::test]
async fn public_keyword_classifier_falls_back_to_general_observation() {
    let classifier = KeywordTaxonomyClassifier::new(DomainV1::System);
    let output = classifier
        .classify(&ClassificationInput::new(
            "misc#1",
            "obscure fragment without known keywords",
        ))
        .await
        .expect("keyword classifier should still return fallback");

    assert_eq!(output.taxonomy.domain, DomainV1::System);
    assert_eq!(output.taxonomy.topic, TopicV1::General);
    assert_eq!(output.taxonomy.aspect, AspectV1::General);
    assert_eq!(output.taxonomy.kind, KindV1::Observation);
    assert!(output.needs_review);
}

#[tokio::test]
async fn public_keyword_classifier_can_explain_keyword_hits() {
    let classifier = KeywordTaxonomyClassifier::default();
    let (_output, report) = classifier
        .classify_with_report(&ClassificationInput::new(
            "roadmap#phase9",
            "Use lexical-first retrieval as the baseline decision because explainability matters.",
        ))
        .await
        .expect("keyword classifier should produce report");

    assert!(report.topic_hits.contains(&"retrieval".to_string()));
    assert!(report.kind_hits.contains(&"decision".to_string()));
}

#[test]
fn public_classification_input_from_record_includes_structured_metadata() {
    use agent_memos::memory::record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        TruthLayer, ValidityWindow,
    };

    let record = MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-19T00:00:00Z".to_string(),
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: "Use lexical-first as baseline.".to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    };

    let input = ClassificationInput::from_record(&record);
    assert_eq!(input.source_kind.as_deref(), Some("note"));
    assert_eq!(input.scope.as_deref(), Some("project"));
    assert_eq!(input.record_type.as_deref(), Some("observation"));
}

#[tokio::test]
async fn public_keyword_classifier_allows_review_threshold_override() {
    let classifier = KeywordTaxonomyClassifier::default().with_review_threshold(1.1);
    let output = classifier
        .classify(&ClassificationInput::new(
            "roadmap#phase9",
            "Use lexical-first retrieval as the baseline decision because explainability matters.",
        ))
        .await
        .expect("keyword classifier should classify");

    assert!(output.needs_review);
}

#[tokio::test]
async fn public_keyword_classifier_supports_memory_record_entrypoint() {
    use agent_memos::memory::record::{
        MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
        TruthLayer, ValidityWindow,
    };

    let record = MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-19T00:00:00Z".to_string(),
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text:
            "Use lexical-first retrieval as the baseline decision because explainability matters."
                .to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    };

    let classifier = KeywordTaxonomyClassifier::default();
    let output = classifier
        .classify_record(&record)
        .await
        .expect("record entrypoint should classify");

    assert_eq!(output.taxonomy.topic, TopicV1::Retrieval);
}
