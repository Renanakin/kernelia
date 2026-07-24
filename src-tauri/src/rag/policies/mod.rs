use crate::rag::models::{DecisionMode, RiskLevel};

#[derive(Debug, Clone)]
pub struct PolicyHint {
    pub tool_name: String,
    pub risk_level: RiskLevel,
    pub decision_mode: DecisionMode,
    pub requires_snapshot: bool,
}

pub fn default_policy_hints() -> Vec<PolicyHint> {
    Vec::new()
}
