use crate::core::session::SessionData;
use crate::core::tool::{Tool, ToolError, ToolInput, ToolOutput, ToolParam};
use async_trait::async_trait;
use serde_json::json;

pub struct RethinkTool;

#[async_trait]
impl Tool for RethinkTool {
    fn name(&self) -> &str {
        "再度思考"
    }

    fn description(&self) -> &str {
        "将当前的对话历史再次输入到模型中，让模型继续思考更多的可能性。适合在模型单次达到输出量限制时使用。"
    }

    fn parameters(&self) -> Vec<ToolParam> {
        vec![ToolParam {
            name: "理由".to_string(),
            description: "调用这个工具的理由".to_string(),
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
            .get("rethink")
            .map(|config| config.auto_approve)
            .unwrap_or(true)
        {
            return Err(ToolError::NotAuthorized);
        }

        // 添加工具调用消息到会话
        session.add_message(
            crate::core::session::Role::System,
            format!(
                "[{}]执行成功。理由：{}",
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
            result: json!({"status": "success"}),
        })
    }
}
