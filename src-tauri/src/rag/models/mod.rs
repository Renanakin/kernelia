use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryCategory {
    Specific,
    Short,
    Ambiguous,
    SymptomBased,
    ActionRequest,
    UnsafeRequest,
    OutOfDomain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DomainSpecialty {
    System,
    Telemetry,
    Network,
    Processes,
    Services,
    Maintenance,
    Security,
    Drivers,
    Filesystem,
    Audit,
    Performance,
    Software,
    SensitiveOps,
    Megaboss,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMode {
    Clarify,
    Explain,
    Simulate,
    Execute,
    Escalate,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    R0,
    R1,
    R2,
    R3,
    R4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    pub normalized_text: String,
    pub query_category: QueryCategory,
    pub specialty: DomainSpecialty,
    pub urgency: String,
    pub symptoms: Vec<String>,
    pub entities: Vec<String>,
    pub ambiguity_score: f32,
    pub requires_clarification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalHit {
    pub source_type: String,
    pub source_id: String,
    pub title: String,
    pub score_lexical: f32,
    pub score_vector: f32,
    pub score_final: f32,
    pub specialty: DomainSpecialty,
    pub entity_key: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAssessment {
    pub level: ConfidenceLevel,
    pub score: f32,
    pub reason_codes: Vec<String>,
    pub should_use_context: bool,
    pub should_ask_clarifying_question: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEnvelope {
    pub query_category: QueryCategory,
    pub specialty: DomainSpecialty,
    pub confidence_level: ConfidenceLevel,
    pub confidence_score: f32,
    pub risk_level: RiskLevel,
    pub decision_mode: DecisionMode,
    pub requires_clarification: bool,
    pub requires_live_state: bool,
    pub requires_snapshot: bool,
    pub requires_human: bool,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    pub enabled: bool,
    pub db_filename: String,
    pub migrations_dir: String,
    pub seeds_dir: String,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            db_filename: "kernelia_rag.db".to_string(),
            migrations_dir: "migrations".to_string(),
            seeds_dir: "seeds".to_string(),
        }
    }
}
