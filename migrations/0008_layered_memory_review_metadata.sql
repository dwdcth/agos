ALTER TABLE fact_dsl_records
    ADD COLUMN classification_confidence REAL;

ALTER TABLE fact_dsl_records
    ADD COLUMN needs_review INTEGER NOT NULL DEFAULT 0;
