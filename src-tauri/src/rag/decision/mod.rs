use crate::rag::models::{ConfidenceAssessment, DecisionEnvelope, QueryAnalysis};
use crate::rag::retrieval::RetrievalBundle;

#[derive(Debug, Clone)]
pub struct DecisionContext {
    pub analysis: QueryAnalysis,
    pub retrieval: RetrievalBundle,
    pub confidence: ConfidenceAssessment,
    pub decision: DecisionEnvelope,
}
