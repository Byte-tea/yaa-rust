pub mod commands;

use crate::core::session::{SessionData, Message, FinishReason};
use crate::core::tool::ToolRegistry;
use crate::agent::{process_session, api::OpenAIClient};
use anyhow::Result;
use std::io::{self, Write};

pub async fn run_interactive(tool_registry: &ToolRegistry, client: &OpenAIClient) -> Result<()> {
    println!("=== YAA 智能体命令行交互模式 ===");
    println!("输入 '/exit' 退出");
    
    let mut session = SessionData::default();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.eq_ignore_ascii_case("/exit") {
            break;
        }
        
        session.messages.push(Message {
            role: crate::core::session::Role::User,
            content: input.to_string(),
        });
        
        let response = process_session(session.clone(), tool_registry, client).await?;
        
        for msg in &response.messages {
            println!("【{:?}】\n{}", msg.role, msg.content);
            session.add_message(msg.role.clone(), msg.content.clone());
        }
        
        if matches!(response.finish_reason, FinishReason::Completed | FinishReason::Interrupted) {
            break;
        }
    }
    
    Ok(())
}