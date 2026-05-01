CREATE VIRTUAL TABLE IF NOT EXISTS memory_records_fts USING fts5(
    source_label,
    content_text,
    content = 'memory_records',
    content_rowid = 'rowid',
    tokenize = 'simple'
);

CREATE TRIGGER IF NOT EXISTS memory_records_ai AFTER INSERT ON memory_records BEGIN
    INSERT INTO memory_records_fts(rowid, source_label, content_text)
    VALUES (new.rowid, COALESCE(new.source_label, ''), new.content_text);
END;

CREATE TRIGGER IF NOT EXISTS memory_records_ad AFTER DELETE ON memory_records BEGIN
    INSERT INTO memory_records_fts(memory_records_fts, rowid, source_label, content_text)
    VALUES ('delete', old.rowid, COALESCE(old.source_label, ''), old.content_text);
END;

CREATE TRIGGER IF NOT EXISTS memory_records_au AFTER UPDATE ON memory_records BEGIN
    INSERT INTO memory_records_fts(memory_records_fts, rowid, source_label, content_text)
    VALUES ('delete', old.rowid, COALESCE(old.source_label, ''), old.content_text);
    INSERT INTO memory_records_fts(rowid, source_label, content_text)
    VALUES (new.rowid, COALESCE(new.source_label, ''), new.content_text);
END;

INSERT INTO memory_records_fts(memory_records_fts) VALUES ('rebuild');
