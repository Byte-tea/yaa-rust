mod agent;
mod cli;
mod core;
use actix_web::{web, App, HttpServer};
use agent::{process_session, api::OpenAIClient};
use clap::{Parser, Subcommand};
use core::{session::SessionData, tool::ToolRegistry};
use agent::tools::finish::FinishTool;
use agent::tools::rethink::RethinkTool;
use cli::commands::{Commands, handle_command};
use tokio;

use std::fs;
use anyhow::{Context, Result};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// 启动服务监听模式
    #[arg(long)]
    serve: bool,

    /// 指定服务监听端口
    #[arg(long, default_value_t = 12345)]
    port: u16,

    /// 指定配置文件路径
    #[arg(long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn load_config(path: Option<&str>) -> Result<SessionData> {
    // 创建默认配置
    let mut session_data = SessionData::default();
    
    // 如果指定了配置文件，则加载并深度合并用户配置
    if let Some(config_path) = path {
        let config_content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path))?;
        
        // 反序列化用户配置
        let user_config: serde_json::Value = serde_json::from_str(&config_content)
            .with_context(|| format!("Failed to parse config file: {}", config_path))?;
        
        // 将默认配置转换为Value以便合并
        let mut default_config = serde_json::to_value(&session_data.config)?;
        
        // 深度合并配置
        if let Some(user_config) = user_config.get("config") {
            merge_json(&mut default_config, user_config);
        }
        
        // 转换回Config结构体
        session_data.config = serde_json::from_value(default_config)?;
    }
    
    Ok(session_data)
}

fn merge_json(target: &mut serde_json::Value, source: &serde_json::Value) {
    match (target, source) {
        (serde_json::Value::Object(target), serde_json::Value::Object(source)) => {
            for (key, value) in source {
                if let Some(target_value) = target.get_mut(key) {
                    merge_json(target_value, value);
                } else {
                    target.insert(key.clone(), value.clone());
                }
            }
        }
        (target, source) => {
            *target = source.clone();
        }
    }
}

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
    let args = Args::parse();
    let session_data = load_config(args.config.as_deref())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if args.serve {
        println!("启动服务监听模式，端口: {}", args.port);
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(session_data.clone()))
                .service(web::resource("/").route(web::post().to(handle_request)))
        })
        .bind(format!("0.0.0.0:{}", args.port))?
        .run()
        .await
    } else {
        let mut tool_registry = ToolRegistry::new();
        tool_registry.register(FinishTool);
        tool_registry.register(RethinkTool);

        let client = OpenAIClient::new(
            session_data.config.llm_api.provider.api_key.to_string(),
            Some(session_data.config.llm_api.provider.api_url.to_string()),
        );
        
        if let Some(cmd) = args.command {
            handle_command(cmd).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        } else {
            cli::run_interactive(&tool_registry, &client).await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }
        Ok(())
    }
}