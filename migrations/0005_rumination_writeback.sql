CREATE TABLE IF NOT EXISTS spq_queue_items (
    item_id TEXT PRIMARY KEY,
    trigger_kind TEXT NOT NULL CHECK (
        trigger_kind IN ('action_failure', 'user_correction', 'metacog_veto')
    ),
    status TEXT NOT NULL CHECK (
        status IN ('queued', 'claimed', 'completed', 'failed', 'cancelled')
    ),
    subject_ref TEXT NOT NULL,
    dedupe_key TEXT NOT NULL,
    cooldown_key TEXT NOT NULL,
    budget_bucket TEXT NOT NULL,
    priority INTEGER NOT NULL,
    budget_cost INTEGER NOT NULL DEFAULT 1 CHECK (budget_cost >= 0),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    cooldown_until TEXT,
    next_eligible_at TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    evidence_refs_json TEXT,
    source_report_json TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    processed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_spq_queue_items_ready
    ON spq_queue_items(status, next_eligible_at, priority DESC, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_spq_queue_items_dedupe
    ON spq_queue_items(dedupe_key, status, created_at DESC);

CREATE TABLE IF NOT EXISTS lpq_queue_items (
    item_id TEXT PRIMARY KEY,
    trigger_kind TEXT NOT NULL CHECK (
        trigger_kind IN (
            'session_boundary',
            'evidence_accumulation',
            'idle_window',
            'abnormal_pattern_accumulation'
        )
    ),
    status TEXT NOT NULL CHECK (
        status IN ('queued', 'claimed', 'completed', 'failed', 'cancelled')
    ),
    subject_ref TEXT NOT NULL,
    dedupe_key TEXT NOT NULL,
    cooldown_key TEXT NOT NULL,
    budget_bucket TEXT NOT NULL,
    priority INTEGER NOT NULL,
    budget_cost INTEGER NOT NULL DEFAULT 1 CHECK (budget_cost >= 0),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK (attempt_count >= 0),
    cooldown_until TEXT,
    next_eligible_at TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    evidence_refs_json TEXT,
    source_report_json TEXT,
    last_error TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    processed_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_lpq_queue_items_ready
    ON lpq_queue_items(status, next_eligible_at, priority DESC, created_at ASC);

CREATE INDEX IF NOT EXISTS idx_lpq_queue_items_dedupe
    ON lpq_queue_items(dedupe_key, status, created_at DESC);

CREATE TABLE IF NOT EXISTS rumination_trigger_state (
    queue_tier TEXT NOT NULL CHECK (queue_tier IN ('spq', 'lpq')),
    trigger_kind TEXT NOT NULL,
    dedupe_key TEXT NOT NULL,
    cooldown_key TEXT NOT NULL,
    budget_bucket TEXT NOT NULL,
    budget_window_started_at TEXT,
    budget_spent INTEGER NOT NULL DEFAULT 0 CHECK (budget_spent >= 0),
    cooldown_until TEXT,
    last_enqueued_at TEXT,
    last_seen_at TEXT NOT NULL,
    last_decision TEXT NOT NULL CHECK (
        last_decision IN ('enqueued', 'deduped', 'cooldown_blocked', 'budget_blocked', 'ignored')
    ),
    last_item_id TEXT,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (queue_tier, dedupe_key)
);

CREATE INDEX IF NOT EXISTS idx_rumination_trigger_state_dedupe
    ON rumination_trigger_state(queue_tier, dedupe_key, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_rumination_trigger_state_cooldown
    ON rumination_trigger_state(queue_tier, cooldown_key, cooldown_until);

CREATE TABLE IF NOT EXISTS local_adaptation_entries (
    entry_id TEXT PRIMARY KEY,
    subject_ref TEXT NOT NULL,
    target_kind TEXT NOT NULL CHECK (
        target_kind IN ('self_state', 'risk_boundary', 'private_t3')
    ),
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    source_queue_item_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_local_adaptation_entries_subject
    ON local_adaptation_entries(subject_ref, target_kind, updated_at DESC);

CREATE TABLE IF NOT EXISTS rumination_candidates (
    candidate_id TEXT PRIMARY KEY,
    source_queue_item_id TEXT,
    candidate_kind TEXT NOT NULL CHECK (
        candidate_kind IN ('skill_template', 'promotion_candidate', 'value_adjustment_candidate')
    ),
    subject_ref TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    evidence_refs_json TEXT,
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'consumed', 'rejected', 'archived')
    ),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_rumination_candidates_kind_status
    ON rumination_candidates(candidate_kind, status, updated_at DESC);
