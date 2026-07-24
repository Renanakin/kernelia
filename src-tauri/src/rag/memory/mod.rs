#[derive(Debug, Clone)]
pub struct MemoryFactDraft {
    pub fact_type: String,
    pub fact_key: String,
    pub fact_value: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Default)]
pub struct MemorySnapshotDraft {
    pub summary: String,
    pub facts: Vec<MemoryFactDraft>,
    pub tags: Vec<String>,
    pub component_states: Vec<(String, String)>,
    pub open_questions: Vec<String>,
    pub tool_actions: Vec<String>,
    pub resolved: bool,
}
