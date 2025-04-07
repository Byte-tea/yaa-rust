use crate::core::session::SessionData;
use crate::core::tool::{Tool, ToolError, ToolInput, ToolOutput, ToolParam};
use async_trait::async_trait;
use serde_json::json;

pub struct FinishTool;

#[async_trait]
impl Tool for FinishTool {
    fn name(&self) -> &str {
        "完成会话"
    }

    fn description(&self) -> &str {
        "当任务完成时调用此工具结束会话。需要提供任务完成的总结理由。"
    }

    fn parameters(&self) -> Vec<ToolParam> {
        vec![ToolParam {
            name: "理由".to_string(),
            description: "任务完成的总结理由".to_string(),
            required: true,
            r#type: "string".to_string(),
        }]
    }

    fn default_config(&self) -> crate::core::session::ToolApprovalConfig {
        crate::core::session::ToolApprovalConfig { auto_approve: true }
    }

    async fn execute(
        &self,
        input: ToolInput,
        session: &mut SessionData,
    ) -> Result<ToolOutput, ToolError> {
        // 检查自动授权配置
        if !session
            .config
            .tool
            .tools
            .get("finish")
            .map(|config| config.auto_approve)
            .unwrap_or(true)
        {
            return Err(ToolError::NotAuthorized);
        }

        // 修改会话状态为已完成
        session.status = crate::core::session::SessionStatus::Completed;

        // 添加工具调用消息到会话
        session.add_message(
            crate::core::session::Role::System,
            format!(
                "[{}]执行成功。任务完成理由：{}",
                self.name(),
                input
                    .param_values
                    .get("理由")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未提供理由")
            ),
        );

        Ok(ToolOutput {
            name: self.name().to_string(),
            result: json!({"status": "completed"}),
        })
    }
}
