use agent_memos::memory::classifier::{ClassificationInput, KeywordTaxonomyClassifier};

#[tokio::test]
async fn public_keyword_classifier_report_surfaces_keyword_hits() {
    let classifier = KeywordTaxonomyClassifier::default();
    let (_output, report) = classifier
        .classify_with_report(&ClassificationInput::new(
            "roadmap#phase9",
            "Use lexical-first retrieval as the baseline decision because explainability matters.",
        ))
        .await
        .expect("keyword classifier should produce report");

    assert!(report.domain_hits.contains(&"roadmap".to_string()));
    assert!(report.topic_hits.contains(&"retrieval".to_string()));
    assert!(report.aspect_hits.contains(&"use".to_string()));
    assert!(report.kind_hits.contains(&"decision".to_string()));
}
