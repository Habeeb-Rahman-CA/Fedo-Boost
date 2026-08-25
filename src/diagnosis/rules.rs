#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    Warning,
    Healthy,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DiagnosticIssue {
    pub severity: IssueSeverity,
    pub category: String,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub fixable_action: Option<String>,
}
