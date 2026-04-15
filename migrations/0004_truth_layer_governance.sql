CREATE TABLE IF NOT EXISTS truth_t3_state (
    record_id TEXT PRIMARY KEY REFERENCES memory_records(id) ON DELETE CASCADE,
    confidence TEXT NOT NULL CHECK (confidence IN ('low', 'medium', 'high')),
    revocation_state TEXT NOT NULL CHECK (revocation_state IN ('active', 'revoked')),
    revoked_at TEXT,
    revocation_reason TEXT,
    shared_conflict_note TEXT,
    last_reviewed_at TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_truth_t3_state_revocation_state
    ON truth_t3_state(revocation_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_truth_t3_state_confidence
    ON truth_t3_state(confidence, updated_at DESC);

INSERT INTO truth_t3_state (
    record_id,
    confidence,
    revocation_state,
    revoked_at,
    revocation_reason,
    shared_conflict_note,
    last_reviewed_at
)
SELECT
    id,
    'medium',
    'active',
    NULL,
    NULL,
    NULL,
    NULL
FROM memory_records
WHERE truth_layer = 't3'
  AND id NOT IN (SELECT record_id FROM truth_t3_state);

CREATE TABLE IF NOT EXISTS truth_promotion_reviews (
    review_id TEXT PRIMARY KEY,
    source_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    target_layer TEXT NOT NULL CHECK (target_layer IN ('t2')),
    result_trigger_state TEXT NOT NULL CHECK (result_trigger_state IN ('pending', 'passed', 'rejected')),
    evidence_review_state TEXT NOT NULL CHECK (evidence_review_state IN ('pending', 'passed', 'rejected')),
    consensus_check_state TEXT NOT NULL CHECK (consensus_check_state IN ('pending', 'passed', 'rejected')),
    metacog_approval_state TEXT NOT NULL CHECK (metacog_approval_state IN ('pending', 'passed', 'rejected')),
    decision_state TEXT NOT NULL CHECK (decision_state IN ('pending', 'approved', 'rejected', 'cancelled')),
    review_notes_json TEXT,
    approved_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_truth_promotion_reviews_source
    ON truth_promotion_reviews(source_record_id, decision_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_truth_promotion_reviews_decision
    ON truth_promotion_reviews(decision_state, updated_at DESC);

CREATE TABLE IF NOT EXISTS truth_promotion_evidence (
    review_id TEXT NOT NULL REFERENCES truth_promotion_reviews(review_id) ON DELETE CASCADE,
    evidence_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    evidence_role TEXT NOT NULL CHECK (evidence_role IN ('supporting', 'contradicting', 'context')),
    evidence_note_json TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (review_id, evidence_record_id)
);

CREATE INDEX IF NOT EXISTS idx_truth_promotion_evidence_role
    ON truth_promotion_evidence(review_id, evidence_role, created_at DESC);

CREATE TABLE IF NOT EXISTS truth_ontology_candidates (
    candidate_id TEXT PRIMARY KEY,
    source_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    basis_record_ids_json TEXT NOT NULL,
    proposed_structure_json TEXT NOT NULL,
    time_stability_state TEXT NOT NULL CHECK (time_stability_state IN ('pending', 'passed', 'rejected')),
    agent_reproducibility_state TEXT NOT NULL CHECK (agent_reproducibility_state IN ('pending', 'passed', 'rejected')),
    context_invariance_state TEXT NOT NULL CHECK (context_invariance_state IN ('pending', 'passed', 'rejected')),
    predictive_utility_state TEXT NOT NULL CHECK (predictive_utility_state IN ('pending', 'passed', 'rejected')),
    structural_review_state TEXT NOT NULL CHECK (structural_review_state IN ('pending', 'passed', 'rejected')),
    candidate_state TEXT NOT NULL CHECK (candidate_state IN ('pending', 'accepted', 'rejected', 'withdrawn')),
    decided_at TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_truth_ontology_candidates_state
    ON truth_ontology_candidates(candidate_state, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_truth_ontology_candidates_source
    ON truth_ontology_candidates(source_record_id, candidate_state, updated_at DESC);
