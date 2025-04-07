use super::session::{SessionData, ToolApprovalConfig};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Tool validation failed: {0}")]
    ValidationFailed(String),
    #[error("Tool not found")]
    NotFound,
    #[error("Tool not authorized")]
    NotAuthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInput {
    pub name: String,
    pub params: Vec<ToolParam>,
    #[serde(default)]
    pub param_values: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub name: String,
    pub result: serde_json::Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Vec<ToolParam>;
    fn default_config(&self) -> super::session::ToolApprovalConfig;

    async fn execute(
        &self,
        input: ToolInput,
        session: &mut SessionData,
    ) -> Result<ToolOutput, ToolError>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    tools_config: HashMap<String, super::session::ToolApprovalConfig>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
            tools_config: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: impl Tool + 'static) {
        let name = tool.name().to_string();
        let config = tool.default_config();
        self.tools.insert(name.clone(), Arc::new(tool));
        self.tools_config.insert(name, config);
    }

    pub async fn execute(
        &self,
        tool_name: &str,
        input: ToolInput,
        session: &mut SessionData,
    ) -> Result<ToolOutput, ToolError> {
        let tool = self.tools.get(tool_name).ok_or(ToolError::NotFound)?;

        // 检查工具授权配置
        let auto_approve = self
            .tools_config
            .get(tool_name)
            .map(|config| config.auto_approve)
            .unwrap_or(true); // 默认允许

        if !auto_approve {
            return Err(ToolError::NotAuthorized);
        }

        tool.execute(input, session).await
    }

    pub fn get_tools_config(&self) -> &std::collections::HashMap<String, ToolApprovalConfig> {
        &self.tools_config
    }

    pub fn list_tools(&self) -> Vec<Arc<dyn Tool>> {
        self.tools.values().cloned().collect()
    }

    pub fn has_tool(&self, tool_name: &str) -> bool {
        self.tools.contains_key(tool_name)
    }
}
