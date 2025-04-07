mod agent;
mod core;
use actix_web::{web, App, HttpServer};
use agent::{process_session, api::OpenAIClient};
use core::{session::SessionData, tool::ToolRegistry};
use agent::tools::finish::FinishTool;
use agent::tools::rethink::RethinkTool;
use tokio;

async fn handle_request(session_data: web::Json<SessionData>) -> actix_web::HttpResponse {
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(FinishTool);
    tool_registry.register(RethinkTool);

    let client = OpenAIClient::new(
        session_data.config.llm_api.provider.api_key.to_string(),
        Some(session_data.config.llm_api.provider.api_url.to_string()),
    );

    match process_session(session_data.into_inner(), &tool_registry, &client).await {
        Ok(response) => {
            match serde_json::to_string(&response) {
                Ok(json) => actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json),
                Err(e) => actix_web::HttpResponse::InternalServerError()
                    .body(format!("Failed to serialize response: {}", e))
            }
        },
        Err(e) => {
            eprintln!("Agent Error: {}", e);
            actix_web::HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 启动HTTP服务器
    HttpServer::new(|| {
        App::new()
            .service(web::resource("/api/process").route(web::post().to(handle_request)))
    })
    .bind("0.0.0.0:12345")?
    .run()
    .await
}