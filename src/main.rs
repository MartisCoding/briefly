

mod config;
mod logging;
mod llm_client;
mod scoring;
mod analysis_result;

use log::{info, error};
use actix_web::{web, App, HttpServer, HttpResponse};
use config::Config;
use llm_client::LLMClient;

use crate::analysis_result::AnalysisRequest;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logging::init_logger();
    info!("Starting Briefly server...");
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration from environment: {}", e);
            std::process::exit(1);
        }
    };
    info!("Configuration loaded: {}", config);
    let llm_client = web::Data::new(LLMClient::new(
        config.llm_api_key.clone(),
        config.llm_base_url.clone(),
        config.llm_model.clone(),
        config.llm_enable_reasoning,
    ));
    HttpServer::new(move || {
        App::new()
            .app_data(llm_client.clone())
            .wrap(actix_web::middleware::Logger::default())
            .route("/analyze", web::post().to(analyze_handler))
            .route("/ping", web::get().to(ping))
            //.route("/complete", web::post().to(complete_handler))
            //.route("/score", web::post().to(score_handler))
            //.route("/verdict", web::post().to(verdict_handler))
    })
    .bind((config.server_host.as_str(), config.server_port))?
    .run()
    .await
}

async fn analyze_handler(llm_client: web::Data<LLMClient>, body: AnalysisRequest) -> HttpResponse {
    let llm_issues = llm_client.analyze_input(&body.text).await;
    match llm_issues {
        Ok(issues) => {
            let transformed_issues: Vec<analysis_result::Issue> = issues.into_iter()
                .filter_map(|issue| analysis_result::Issue::from_llm_issue(issue, &body.text, &body.filter))
                .collect();
            let analysis_result = analysis_result::AnalysisResult {
                issues: transformed_issues,
                //scoring: scoring::score(&transformed_issues),
            };
            HttpResponse::Ok().json(analysis_result)
        },
        Err(e) => {
            error!("LLM analysis failed: {}", e);
            return HttpResponse::InternalServerError().body("LLM analysis failed");
        }
    }
}

async fn ping() -> HttpResponse {
    HttpResponse::Ok().body("Pong")
}

