# 项目架构

yaa 智能体采用 Rust 实现，支持 WASM（WebAssembly）和 WASI 运行时环境，可作为服务器模式或命令行交互模式运行，也可编译为 WASM 被 HTML 调用。

## 实现

### 核心功能

1. 会话数据管理 (SessionData)
2. 工具调用框架 (ToolRegistry)
3. 提示词生成器 (PromptGenerator)
4. 大语言模型 API 集成 (OpenAIClient)
5. 智能体核心逻辑 (process_session)
6. 工具实现：
   - 再度思考工具 (RethinkTool)
   - 完成会话工具 (FinishTool)

### 代码结构

```mermaid
%% yaa 实现代码结构图
classDiagram
    class SessionData {
        +id: String
        +title: Option<String>
        +start_time: String
        +character: String
        +status: String
        +messages: Vec<Message>
        +config: Config
        
        +new(character: String) -> Self
        +add_message(role: Role, content: String)
    }

    class ToolRegistry {
        +tools: HashMap<String, Tool>
        
        +new() -> Self
        +execute(name: String, params: Value) -> Result<Value>
    }

    class PromptGenerator {
        +generate(session_data: &SessionData, tools: &[ToolInfo]) -> String
        +generate_with_current_time(session_data: &SessionData, tools: &[ToolInfo]) -> String
    }

    class OpenAIClient {
        +api_key: String
        +base_url: Option<String>
        
        +new(api_key: String, base_url: Option<String>) -> Self
        +chat_completion(messages: Vec<Message>) -> Result<String>
    }

    class Agent {
        +process_session(session_data: SessionData) -> Result<AgentResponseData>
    }

    class RethinkTool {
        +name() -> &str
        +execute() -> Result<ToolOutput>
    }

    class FinishTool {
        +name() -> &str
        +execute() -> Result<ToolOutput>
    }

    SessionData --> Message : 包含
    SessionData --> Config : 包含
    Agent --> SessionData : 处理
    Agent --> ToolRegistry : 使用
    Agent --> PromptGenerator : 使用
    Agent --> OpenAIClient : 使用
    ToolRegistry --> RethinkTool : 注册
    ToolRegistry --> FinishTool : 注册
    ToolRegistry --> ToolInput : 处理
    ToolRegistry --> ToolOutput : 生成
```

### 核心工作流程

1. 接收会话数据 (SessionData)
2. 检查并补全会话配置
3. 生成提示词 (PromptGenerator)
4. 调用大语言模型 API (OpenAIClient)
5. 解析工具调用 (extract_tool_call)
6. 检查工具授权：
   - 检查会话配置中的 auto_approve 设置
   - 检查用户消息中的授权标记
7. 执行工具 (ToolRegistry)
8. 生成响应数据 (AgentResponseData/StreamResponseData)

```mermaid
%% yaa 智能体工作流程图
sequenceDiagram
    客户端->>+智能体: 发送 SessionData
    智能体->>+提示词生成器: 生成提示词
    提示词生成器-->>-智能体: 返回提示词
    智能体->>+大语言模型API: 发送提示词
    大语言模型API-->>-智能体: 返回响应
    智能体->>+工具解析器: 解析工具调用
    工具解析器-->>-智能体: 返回工具名和参数
    智能体->>+授权检查: 验证工具调用权限
    授权检查-->>-智能体: 返回授权结果
    智能体->>+工具注册表: 执行工具
    工具注册表-->>-智能体: 返回工具结果
    智能体-->>-客户端: 返回 AgentResponseData
```

### 工具实现

#### 再度思考工具 (RethinkTool)

- 功能：将当前的对话历史再次输入到模型中，让模型继续思考更多的可能性
- 参数：
  - 理由 (string, required): 调用这个工具的理由
- 默认配置：自动授权 (auto_approve: true)

#### 完成会话工具 (FinishTool)

- 功能：当任务完成时调用此工具结束会话
- 参数：
  - 理由 (string, required): 任务完成的总结理由
- 默认配置：自动授权 (auto_approve: true)

### 数据结构

#### 会话数据 (SessionData)

```rust
pub struct SessionData {
    pub id: String,
    pub title: Option<String>,
    pub start_time: String,
    pub character: String,
    pub status: String,
    pub messages: Vec<Message>,
    pub config: Config,
}

pub struct Message {
    pub role: Role, // user|assistant|system|tool|error
    pub content: String,
}

pub struct Config {
    pub yaa: YaaConfig,
    pub llm_api: LlmApiConfig,
    pub tool: ToolConfig,
}
```

#### 智能体响应数据

```rust
pub struct AgentResponseData {
    pub id: String,
    pub title: Option<String>,
    pub start_time: String,
    pub finish_reason: String,
    pub messages: Vec<Message>,
    pub usage: Usage,
}

pub struct StreamResponseData {
    pub id: String,
    pub status: String,
    pub message: Message,
}
```

### 已知问题

1. 消息角色处理问题：
   - 当消息角色为 Tool 时调用 API，API 返回的数据无法格式化为 JSON
   - 其他非 User 和 Assistant 的消息角色也可能导致类似问题

### 待完成功能

1. WASM 支持 (web/模块)
   - WASM 绑定 (bindings.rs)
   - Web Worker 支持 (worker.rs)

2. 命令行模式 (cli/模块)
   - 命令行入口 (mod.rs)
   - 子命令实现 (commands.rs)

## 流程分析

- [流程分析](FlowAnalyze/FlowAnalyze.md)

## 新工具

- [贡献新工具](new_tools.md)
