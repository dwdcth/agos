use serde::Serialize;

use crate::cognition::{
    action::{ActionBranch, ActionCandidate, ActionKind},
    report::{DecisionReport, GateReport, ScoredBranchReport},
    value::{ProjectedScore, ScoredBranch, ValueConfig, ValueVector},
    working_memory::{MetacognitiveFlag, WorkingMemory},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GateDecision {
    Warning,
    SoftVeto,
    HardVeto,
    Escalate,
}

#[derive(Debug, Clone)]
pub struct MetacognitionService {
    minimum_supporting_evidence: usize,
    escalate_flag_codes: Vec<&'static str>,
    soft_veto_risk_markers: Vec<&'static str>,
    hard_veto_risk_markers: Vec<&'static str>,
    safe_response: String,
    fallback_regulative_summary: String,
}

impl Default for MetacognitionService {
    fn default() -> Self {
        Self {
            minimum_supporting_evidence: 1,
            escalate_flag_codes: vec!["human_review_required"],
            soft_veto_risk_markers: vec!["clarification_required", "downgrade_required"],
            hard_veto_risk_markers: vec!["unsafe_action", "irreversible_change"],
            safe_response: "pause execution and request a safer alternative".to_string(),
            fallback_regulative_summary: "pause and request clarification".to_string(),
        }
    }
}

impl MetacognitionService {
    pub fn evaluate(
        &self,
        working_memory: &WorkingMemory,
        scored_branches: Vec<ScoredBranch>,
    ) -> DecisionReport {
        let reports = scored_branches
            .iter()
            .cloned()
            .map(ScoredBranchReport::from)
            .collect::<Vec<_>>();
        let mut active_risks = working_memory.present.active_risks.clone();
        let mut metacog_flags = working_memory.present.metacog_flags.clone();
        let top_branch = scored_branches
            .iter()
            .max_by(|left, right| {
                left.projected
                    .final_score
                    .total_cmp(&right.projected.final_score)
            })
            .cloned();

        if let Some(report) = self.escalate_report(&reports, &metacog_flags, &active_risks) {
            return report;
        }

        let Some(top_branch) = top_branch else {
            return DecisionReport {
                scored_branches: reports,
                selected_branch: None,
                gate: GateReport {
                    decision: GateDecision::HardVeto,
                    diagnostics: vec!["no candidate branches available for selection".to_string()],
                    rejected_branch: None,
                    regulative_branch: None,
                    safe_response: Some(self.safe_response.clone()),
                    autonomy_paused: false,
                },
                active_risks,
                metacog_flags,
            };
        };

        if self.has_risk_marker(&top_branch.branch, &self.hard_veto_risk_markers) {
            return DecisionReport {
                scored_branches: reports,
                selected_branch: None,
                gate: GateReport {
                    decision: GateDecision::HardVeto,
                    diagnostics: top_branch
                        .branch
                        .risk_markers
                        .iter()
                        .map(|marker| format!("hard veto triggered by risk marker: {marker}"))
                        .collect(),
                    rejected_branch: Some(ScoredBranchReport::from(top_branch)),
                    regulative_branch: None,
                    safe_response: Some(self.safe_response.clone()),
                    autonomy_paused: false,
                },
                active_risks,
                metacog_flags,
            };
        }

        if self.has_risk_marker(&top_branch.branch, &self.soft_veto_risk_markers) {
            let regulative_branch = scored_branches
                .iter()
                .filter(|branch| branch.branch.candidate.kind == ActionKind::Regulative)
                .max_by(|left, right| {
                    left.projected
                        .final_score
                        .total_cmp(&right.projected.final_score)
                })
                .cloned()
                .unwrap_or_else(|| {
                    self.synthetic_regulative_branch(&top_branch.projected.weight_snapshot)
                });
            let regulative_report = ScoredBranchReport::from(regulative_branch);
            active_risks.push(format!(
                "soft-vetoed branch: {}",
                top_branch.branch.candidate.summary
            ));
            metacog_flags.push(MetacognitiveFlag {
                code: "soft_veto_active".to_string(),
                detail: Some("regulative branch forced into selection".to_string()),
            });

            return DecisionReport {
                scored_branches: reports,
                selected_branch: Some(regulative_report.clone()),
                gate: GateReport {
                    decision: GateDecision::SoftVeto,
                    diagnostics: top_branch
                        .branch
                        .risk_markers
                        .iter()
                        .map(|marker| format!("soft veto triggered by risk marker: {marker}"))
                        .collect(),
                    rejected_branch: Some(ScoredBranchReport::from(top_branch)),
                    regulative_branch: Some(regulative_report),
                    safe_response: None,
                    autonomy_paused: false,
                },
                active_risks,
                metacog_flags,
            };
        }

        let mut diagnostics = Vec::new();
        if top_branch.branch.supporting_evidence.len() < self.minimum_supporting_evidence {
            diagnostics.push(format!(
                "under-supported branch: {}",
                top_branch.branch.candidate.summary
            ));
            active_risks.push(format!(
                "under-supported branch: {}",
                top_branch.branch.candidate.summary
            ));
            metacog_flags.push(MetacognitiveFlag {
                code: "warning_under_supported".to_string(),
                detail: Some("selected branch needs stronger evidence".to_string()),
            });
        }

        DecisionReport {
            scored_branches: reports,
            selected_branch: Some(ScoredBranchReport::from(top_branch)),
            gate: GateReport {
                decision: GateDecision::Warning,
                diagnostics,
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: false,
            },
            active_risks,
            metacog_flags,
        }
    }

    fn escalate_report(
        &self,
        reports: &[ScoredBranchReport],
        metacog_flags: &[MetacognitiveFlag],
        active_risks: &[String],
    ) -> Option<DecisionReport> {
        let diagnostics = metacog_flags
            .iter()
            .filter(|flag| self.escalate_flag_codes.contains(&flag.code.as_str()))
            .map(|flag| match &flag.detail {
                Some(detail) => format!("{}: {detail}", flag.code),
                None => flag.code.clone(),
            })
            .collect::<Vec<_>>();

        if diagnostics.is_empty() {
            return None;
        }

        Some(DecisionReport {
            scored_branches: reports.to_vec(),
            selected_branch: None,
            gate: GateReport {
                decision: GateDecision::Escalate,
                diagnostics,
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: true,
            },
            active_risks: active_risks.to_vec(),
            metacog_flags: metacog_flags.to_vec(),
        })
    }

    fn has_risk_marker(&self, branch: &ActionBranch, markers: &[&str]) -> bool {
        branch
            .risk_markers
            .iter()
            .any(|risk| markers.contains(&risk.as_str()))
    }

    fn synthetic_regulative_branch(&self, weight_snapshot: &ValueConfig) -> ScoredBranch {
        let value = ValueVector {
            goal_progress: 0.35,
            information_gain: 0.35,
            risk_avoidance: 1.0,
            resource_efficiency: 0.45,
            agent_robustness: 1.0,
        };

        ScoredBranch {
            branch: ActionBranch::new(
                ActionCandidate::new(ActionKind::Regulative, &self.fallback_regulative_summary)
                    .with_intent("insert a safe regulating step before continuing"),
            ),
            projected: ProjectedScore {
                final_score: (value.goal_progress * weight_snapshot.goal_progress)
                    + (value.information_gain * weight_snapshot.information_gain)
                    + (value.risk_avoidance * weight_snapshot.risk_avoidance)
                    + (value.resource_efficiency * weight_snapshot.resource_efficiency)
                    + (value.agent_robustness * weight_snapshot.agent_robustness),
                weight_snapshot: weight_snapshot.clone(),
                threshold_passed: true,
            },
            value,
        }
    }
}
