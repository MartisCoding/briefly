use actix_web::FromRequest;
use log::info;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    //pub scoring: Scoring,
    //pub verdict: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    pub start: usize,
    pub end: usize,
    pub message: String, 
    pub severity: Severity,
    pub category: Category,
}

impl Issue {
    pub fn from_llm_issue(llm_issue: crate::llm_client::LLMAnalysisIssue, original_text: &str, filter: &Severity) -> Option<Self> {
        let start = original_text.find(&llm_issue.quote)?;
        let start = original_text[..start].chars().count();
        let end = start + llm_issue.quote.chars().count();
        let severity = match llm_issue.severity.as_str() {
            "info" => {
                if *filter > Severity::Info {
                    return None;
                }
                Severity::Info
            },
            "warn" => {
                if *filter > Severity::Warning {
                    return None;
                }
                Severity::Warning
            },
            "error" => {
                Severity::Error
            },
            _ => return None,
        };
        let category = match llm_issue.category.as_str() {
            "Ambiguity" => Category::Ambiguity,
            "Contradiction" => Category::Contradiction,
            "MissingDetail" => Category::MissingDetail,
            "Risk" => Category::Risk,
            "Question" => Category::Question,
            "Dependency" => Category::Dependency,
            _ => return None,
        };
        info!("Mapped LLM issue: quote='{}', start={}, end={}, message='{}', severity={:?}, category={:?}", llm_issue.quote, start, end, llm_issue.message, severity, category);
        Some(Self {
            start,
            end,
            message: llm_issue.message.clone(),
            severity,
            category,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Test, // Test of request parsing, isn't used in filtering
    Info,
    Warning,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Category {
    Ambiguity,
    Contradiction,
    MissingDetail,
    Risk,
    Question,
    Dependency,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisRequest {
    pub filter: Severity,
    pub text: String,
}

impl FromRequest for AnalysisRequest {
    type Error = actix_web::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;
    
    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let fut = actix_web::web::Json::<AnalysisRequest>::from_request(req, payload);
        Box::pin(async move {
            let json = fut.await?;
            Ok(json.into_inner())
        })
    }
}