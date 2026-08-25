use crate::diagnosis::analyzer::DiagnosisReport;

#[allow(dead_code)]
pub struct RecommendationEngine;

impl RecommendationEngine {
    #[allow(dead_code)]
    pub fn get_quick_fixes(report: &DiagnosisReport) -> Vec<String> {
        let mut fixes = Vec::new();
        for issue in &report.issues {
            fixes.push(format!("[{}] {} -> {}", issue.category, issue.title, issue.recommendation));
        }
        if fixes.is_empty() {
            fixes.push("No immediate action needed. Fedora system is operating efficiently.".to_string());
        }
        fixes
    }
}
