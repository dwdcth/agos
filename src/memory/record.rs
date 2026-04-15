use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub source: SourceRef,
    pub timestamp: RecordTimestamp,
    pub scope: Scope,
    pub record_type: RecordType,
    pub truth_layer: TruthLayer,
    pub provenance: Provenance,
    pub content_text: String,
    pub chunk: Option<ChunkMetadata>,
    pub validity: ValidityWindow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub uri: String,
    pub kind: SourceKind,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Document,
    Conversation,
    Note,
    System,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Conversation => "conversation",
            Self::Note => "note",
            Self::System => "system",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "document" => Some(Self::Document),
            "conversation" => Some(Self::Conversation),
            "note" => Some(Self::Note),
            "system" => Some(Self::System),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordTimestamp {
    pub recorded_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Project,
    Session,
    User,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Session => "session",
            Self::User => "user",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "project" => Some(Self::Project),
            "session" => Some(Self::Session),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordType {
    Observation,
    Decision,
    Fact,
}

impl RecordType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Decision => "decision",
            Self::Fact => "fact",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "observation" => Some(Self::Observation),
            "decision" => Some(Self::Decision),
            "fact" => Some(Self::Fact),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthLayer {
    T1,
    T2,
    T3,
}

impl TruthLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "t1" => Some(Self::T1),
            "t2" => Some(Self::T2),
            "t3" => Some(Self::T3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: String,
    pub imported_via: Option<String>,
    pub derived_from: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub chunk_index: u32,
    pub chunk_count: u32,
    pub anchor: ChunkAnchor,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChunkAnchor {
    CharRange {
        start_char: u32,
        end_char: u32,
    },
    LineRange {
        start_line: u32,
        end_line: u32,
    },
    TurnRange {
        start_turn: u32,
        end_turn: u32,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityWindow {
    pub valid_from: Option<String>,
    pub valid_to: Option<String>,
}
