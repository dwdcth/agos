use crate::{
    ingest::normalize::NormalizedSource,
    memory::record::{ChunkAnchor, ChunkMetadata},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkConfig {
    pub text_char_window: usize,
    pub text_char_overlap: usize,
    pub conversation_turn_window: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            text_char_window: 800,
            text_char_overlap: 100,
            conversation_turn_window: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkDraft {
    pub text: String,
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub anchor: ChunkAnchor,
    pub content_hash: String,
}

pub fn chunk_source(source: &NormalizedSource, config: ChunkConfig) -> Vec<ChunkDraft> {
    if let Some(turns) = &source.conversation_turns {
        let turn_window = config.conversation_turn_window.max(1);
        let mut drafts = Vec::new();

        for (index, batch) in turns.chunks(turn_window).enumerate() {
            let text = batch
                .iter()
                .map(|turn| turn.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let anchor = ChunkAnchor::TurnRange {
                start_turn: batch.first().map(|turn| turn.start_turn).unwrap_or(1),
                end_turn: batch.last().map(|turn| turn.end_turn).unwrap_or(1),
            };
            drafts.push((index as u32, text, anchor));
        }

        return finalize_drafts(drafts);
    }

    finalize_drafts(chunk_text_ranges(
        &source.text,
        config.text_char_window,
        config.text_char_overlap,
    ))
}

fn chunk_text_ranges(text: &str, window: usize, overlap: usize) -> Vec<(u32, String, ChunkAnchor)> {
    let trimmed = text.trim();
    if trimmed.is_empty() || window == 0 {
        return Vec::new();
    }

    let overlap = overlap.min(window.saturating_sub(1));
    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut start = 0usize;
    let mut chunk_index = 0u32;

    while start < chars.len() {
        let mut end = usize::min(start + window, chars.len());

        if end < chars.len()
            && let Some(split) = chars[start..end]
                .iter()
                .rposition(|ch| matches!(ch, '\n' | ' ' | '\t'))
            && split > window / 2
        {
            end = start + split + 1;
        }

        let chunk = chars[start..end]
            .iter()
            .collect::<String>()
            .trim()
            .to_string();
        if !chunk.is_empty() {
            chunks.push((
                chunk_index,
                chunk,
                ChunkAnchor::LineRange {
                    start_line: line_number_at(trimmed, start),
                    end_line: line_number_at(trimmed, end.saturating_sub(1)),
                },
            ));
            chunk_index += 1;
        }

        if end == chars.len() {
            break;
        }

        let next_start = end.saturating_sub(overlap);
        start = if next_start <= start { end } else { next_start };
    }

    chunks
}

fn line_number_at(text: &str, char_index: usize) -> u32 {
    let capped = char_index.min(text.chars().count());
    text.chars().take(capped).filter(|ch| *ch == '\n').count() as u32 + 1
}

fn finalize_drafts(raw: Vec<(u32, String, ChunkAnchor)>) -> Vec<ChunkDraft> {
    let chunk_count = raw.len() as u32;

    raw.into_iter()
        .map(|(chunk_index, text, anchor)| ChunkDraft {
            content_hash: content_hash(&text),
            text,
            chunk_index,
            chunk_count,
            anchor,
        })
        .collect()
}

pub fn to_chunk_metadata(draft: &ChunkDraft) -> ChunkMetadata {
    ChunkMetadata {
        chunk_index: draft.chunk_index,
        chunk_count: draft.chunk_count,
        anchor: draft.anchor.clone(),
        content_hash: draft.content_hash.clone(),
    }
}

fn content_hash(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;

    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("fnv1a64:{hash:016x}")
}
