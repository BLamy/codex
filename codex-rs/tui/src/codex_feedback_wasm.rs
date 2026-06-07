pub const DOCTOR_REPORT_ATTACHMENT_FILENAME: &str = "doctor-report.txt";
pub const FEEDBACK_DIAGNOSTICS_ATTACHMENT_FILENAME: &str = "feedback-diagnostics.txt";
pub const WINDOWS_SANDBOX_LOG_ATTACHMENT_FILENAME: &str = "windows-sandbox-log.txt";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeedbackDiagnostic {
    pub headline: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeedbackDiagnostics {
    diagnostics: Vec<FeedbackDiagnostic>,
}

impl FeedbackDiagnostics {
    pub fn new(diagnostics: Vec<FeedbackDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn diagnostics(&self) -> &[FeedbackDiagnostic] {
        &self.diagnostics
    }

    pub fn attachment_text(&self) -> String {
        self.diagnostics
            .iter()
            .map(|diagnostic| match diagnostic.details.as_deref() {
                Some(details) => format!("{}\n{details}", diagnostic.headline),
                None => diagnostic.headline.clone(),
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeedbackSnapshot {
    feedback_diagnostics: FeedbackDiagnostics,
}

impl FeedbackSnapshot {
    pub fn feedback_diagnostics(&self) -> &FeedbackDiagnostics {
        &self.feedback_diagnostics
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodexFeedback;

impl CodexFeedback {
    pub fn new() -> Self {
        Self
    }

    pub fn snapshot(&self, _thread_id: Option<codex_protocol::ThreadId>) -> FeedbackSnapshot {
        FeedbackSnapshot::default()
    }
}
