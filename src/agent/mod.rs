//! 智能体核心逻辑模块

use crate::agent::api::{Message as ApiMessage, OpenAIApi};
use crate::core::session::{
    AgentResponseData, FinishReason, Message, Role, SessionData, SessionStatus, Usage,
};
use crate::core::tool::{ToolInput, ToolRegistry};
use thiserror::Error;

pub mod api;
pub mod prompt;
pub mod tool;
pub mod tools;

pub use api::{ApiError, OpenAIClient};
pub use prompt::PromptGenerator;
pub use tool::extract_tool_call;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("LLM API error: {0}")]
    LlmApiError(#[from] ApiError),
    #[error("Tool error: {0}")]
    ToolError(String),
    #[error("Invalid session state")]
    InvalidSessionState,
    #[error("No user message found")]
    NoUserMessage,
}

/// 处理会话数据并返回智能体响应
pub async fn process_session(
    mut session_data: SessionData,
    tool_registry: &ToolRegistry,
    llm_client: &OpenAIClient,
) -> Result<AgentResponseData, AgentError> {
    // 检查会话状态
    if matches!(session_data.status, SessionStatus::Completed) {
        return Err(AgentError::InvalidSessionState);
    }

    // 获取最后一条用户消息
    let _last_user_msg = session_data
        .messages
        .iter()
        .rfind(|m| matches!(m.role, Role::User))
        .ok_or(AgentError::NoUserMessage)?;

    loop {
        // 当状态为“已中断”时，很可能是用户授权工具调用，直接跳到工具解析
        if matches!(session_data.status, SessionStatus::InProgress) {
            // 1. 提示词生成（只有当最后一条消息是 user 的时才调用）
            if let Some(last_msg) = session_data.messages.last() {
                if matches!(last_msg.role, Role::User) {
                    let prompt = PromptGenerator::generate_with_current_time(
                        &last_msg.content,
                        &session_data.character,
                        &get_tools_info(tool_registry)[..],
                        &session_data
                            .config
                            .yaa
                            .language
                            .as_deref()
                            .unwrap_or("zh-CN"),
                    );
                    session_data.add_message(Role::System, prompt);
                }
            }

            // 2. 调用大语言模型 API（只有当最后一条消息不是 assistant 的时才调用）
            let _llm_response = if let Some(last_msg) = session_data.messages.last() {
                if !matches!(last_msg.role, Role::Assistant) {
                    let api_messages = session_data
                        .messages
                        .iter()
                        .map(|m| ApiMessage {
                            role: match m.role {
                                Role::User => "user",
                                Role::Assistant => "assistant",
                                Role::System => "system",
                                Role::Tool => "tool",
                                Role::Error => "error",
                            }
                            .to_string(),
                            content: m.content.clone(),
                        })
                        .collect();

                    let llm_response = llm_client
                        .chat_completion(
                            api_messages,
                            &session_data.config.llm_api.provider.model_name,
                            session_data
                                .config
                                .llm_api
                                .provider
                                .model_settings
                                .temperature,
                        )
                        .await?;
                    session_data.add_message(Role::Assistant, &llm_response);
                    llm_response
                } else {
                    // 如果最后一条消息是assistant，则直接使用其内容
                    last_msg.content.clone()
                }
            } else {
                return Err(AgentError::NoUserMessage);
            };
        }

        // 3. 提取工具调用
        let llm_message = session_data
            .messages
            .iter()
            .rev()
            .find(|m| m.role == Role::Assistant)
            .map(|m| &m.content)
            .ok_or(AgentError::InvalidSessionState)?;

        let (tool_name, params, param_values) = match extract_tool_call(&llm_message) {
            Some((name, params, values)) => {
                println!(
                    "[DEBUG] 提取工具调用 - 名称: {:?}, 参数: {:?}, 值: {:?}",
                    name, params, values
                );
                (name.to_string(), params, values)
            }
            None => {
                session_data.add_message(Role::System, "[警告]每次回复至少调用一个工具！");
                continue;
            }
        };

        // 4. 检查工具是否存在
        if !tool_registry.has_tool(&tool_name) {
            session_data.add_message(
                Role::System,
                &format!("[警告]工具\"{}\"不存在！", tool_name),
            );
            continue;
        }

        // 5. 检查工具授权
        let mut user_approved = false;
        if let Some(last_message) = session_data.messages.last() {
            if matches!(last_message.role, Role::User) {
                user_approved = last_message.content.contains(&format!(
                    "<批准>\n<工具名称>{}</工具名称>\n</批准>",
                    tool_name
                ));
            }
        }

        let tool_approved = session_data
            .config
            .tool
            .tools
            .get(&tool_name)
            .map(|config| config.auto_approve)
            .unwrap_or(false)
            || user_approved;

        if !tool_approved {
            session_data.status = SessionStatus::Interrupted;
            break;
        }

        // 6. 执行工具调用

        let _tool_output = tool_registry
            .execute(
                &tool_name,
                ToolInput {
                    name: tool_name.clone(),
                    params,
                    param_values,
                },
                &mut session_data,
            )
            .await
            .map_err(|e| AgentError::ToolError(e.to_string()))?;

        // session_data.add_message(
        //     Role::Tool,
        //     &format!("工具 {} 执行结果: {}", tool_name, tool_output.result),
        // );

        // 5. 检查会话状态
        if !matches!(session_data.status, SessionStatus::InProgress) {
            break;
        }
    }

    // 生成响应数据
    Ok(AgentResponseData {
        id: session_data.id.clone(),
        object: "agent.response".to_string(),
        title: session_data.title.clone(),
        start_time: session_data.start_time,
        finish_reason: match session_data.status {
            SessionStatus::Completed => FinishReason::Completed,
            SessionStatus::Interrupted => FinishReason::Interrupted,
            _ => FinishReason::Failed,
        },
        messages: get_response_messages(&session_data),
        usage: Usage {
            prompt_tokens: 0,     // TODO: 从API响应中获取实际值
            completion_tokens: 0, // TODO: 从API响应中获取实际值
            total_tokens: 0,      // TODO: 从API响应中获取实际值
        },
    })
}

/// 获取工具信息列表
fn get_tools_info(tool_registry: &ToolRegistry) -> Vec<prompt::ToolInfo> {
    tool_registry
        .list_tools()
        .into_iter()
        .map(|tool| prompt::ToolInfo {
            name: tool.name().to_string(),
            description: tool.description().to_string(),
            parameters: serde_json::to_value(tool.parameters())
                .ok()
                .and_then(|v| v.as_array().map(|arr| arr.to_vec()))
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            let obj = v.as_object()?;
                            Some(prompt::Param {
                                name: obj.get("name")?.as_str()?.to_string(),
                                description: obj.get("description")?.as_str()?.to_string(),
                                required: obj.get("required")?.as_bool()?,
                                r#type: obj.get("type")?.as_str()?.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

/// 获取响应消息（最后一条用户消息之后的所有消息）
fn get_response_messages(session_data: &SessionData) -> Vec<Message> {
    let last_user_idx = session_data
        .messages
        .iter()
        .rposition(|m| matches!(m.role, Role::User))
        .unwrap_or(0);

    session_data.messages[last_user_idx + 1..].to_vec()
}
