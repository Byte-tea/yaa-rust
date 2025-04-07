use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponseData {
    pub id: String,
    pub object: String,
    pub title: Option<String>,
    pub start_time: DateTime<Utc>,
    pub finish_reason: FinishReason,
    pub messages: Vec<Message>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamResponseData {
    pub id: String,
    pub object: String,
    pub status: SessionStatus,
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishReason {
    Completed,
    WaitingFeedback,
    Failed,
    Interrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionData {
    pub id: String,
    pub title: Option<String>,
    pub start_time: DateTime<Utc>,
    pub character: String,
    pub status: SessionStatus,
    pub messages: Vec<Message>,
    pub config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    InProgress,
    Interrupted,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub yaa: YaaConfig,
    pub llm_api: LlmApiConfig,
    #[serde(default)]
    pub prompt: PromptConfig,
    #[serde(default)]
    pub tool: ToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct YaaConfig {
    pub stream: bool,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmApiConfig {
    pub provider: ProviderConfig,
    pub stream: bool,
    pub request_timeout: u32,
    pub interval: u32,
    pub retry: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfig {
    pub api_url: String,
    pub api_key: String,
    pub model_name: String,
    pub model_type: ModelType,
    pub cost_per_ktoken: f32,
    pub cost_unit: String,
    pub max_tokens: u32,
    pub model_settings: ModelSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelType {
    pub is_function_call: bool,
    pub is_reasoning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelSettings {
    pub use_custom_temp: bool,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetryConfig {
    pub times: u32,
    pub delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptConfig {
    // 提示词模板配置
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    #[serde(flatten)]
    pub tools: std::collections::HashMap<String, ToolApprovalConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolApprovalConfig {
    pub auto_approve: bool,
}

impl SessionData {
    pub fn new(character: impl Into<String>, tool_registry: Option<&crate::core::tool::ToolRegistry>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: None,
            start_time: Utc::now(),
            character: character.into(),
            status: SessionStatus::InProgress,
            messages: Vec::new(),
            config: Config {
                yaa: YaaConfig::default(),
                llm_api: LlmApiConfig {
                    provider: ProviderConfig {
                        api_url: "https://api.deepseek.com".into(),
                        api_key: String::new(),
                        model_name: "deepseek-chat".into(),
                        model_type: ModelType {
                            is_function_call: true,
                            is_reasoning: true,
                        },
                        cost_per_ktoken: 0.002,
                        cost_unit: "USD".into(),
                        max_tokens: 4096,
                        model_settings: ModelSettings {
                            use_custom_temp: false,
                            temperature: 0.7,
                        },
                    },
                    stream: true,
                    request_timeout: 30,
                    interval: 1000,
                    retry: RetryConfig {
                        times: 3,
                        delay: 1000,
                    },
                },
                prompt: PromptConfig::default(),
                tool: {
                    let mut tools = std::collections::HashMap::new();
                    if let Some(registry) = tool_registry {
                        for (name, config) in registry.get_tools_config() {
                            tools.insert(name.clone(), config.clone());
                        }
                    }
                    ToolConfig { tools }
                },
            },
        }
    }

    pub fn add_message(&mut self, role: Role, content: impl Into<String>) {
        self.messages.push(Message {
            role,
            content: content.into(),
        });
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new("", None)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            yaa: YaaConfig::default(),
            llm_api: LlmApiConfig {
                provider: ProviderConfig {
                    api_url: "https://api.deepseek.com".into(),
                    api_key: String::new(),
                    model_name: "deepseek-chat".into(),
                    model_type: ModelType {
                        is_function_call: true,
                        is_reasoning: true,
                    },
                    cost_per_ktoken: 0.002,
                    cost_unit: "USD".into(),
                    max_tokens: 4096,
                    model_settings: ModelSettings {
                        use_custom_temp: false,
                        temperature: 0.7,
                    },
                },
                stream: true,
                request_timeout: 30,
                interval: 1000,
                retry: RetryConfig {
                    times: 3,
                    delay: 1000,
                },
            },
            prompt: PromptConfig::default(),
            tool: ToolConfig::default(),
        }
    }
}