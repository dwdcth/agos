use serde::Serialize;

use crate::memory::record::{RecordType, Scope, TruthLayer};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SearchFilters {
    pub scope: Option<Scope>,
    pub record_type: Option<RecordType>,
    pub truth_layer: Option<TruthLayer>,
    pub valid_at: Option<String>,
    pub recorded_from: Option<String>,
    pub recorded_to: Option<String>,
}

pub type AppliedFilters = SearchFilters;

impl SearchFilters {
    pub fn scope_value(&self) -> Option<&'static str> {
        self.scope.map(Scope::as_str)
    }

    pub fn record_type_value(&self) -> Option<&'static str> {
        self.record_type.map(RecordType::as_str)
    }

    pub fn truth_layer_value(&self) -> Option<&'static str> {
        self.truth_layer.map(TruthLayer::as_str)
    }
}
