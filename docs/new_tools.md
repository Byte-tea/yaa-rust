# 新增工具开发指南

## 工具创建步骤

1. 在 `src/agent/tools/` 目录下创建新的工具文件，例如 `my_tool.rs`
2. 实现 `Tool` trait 的四个必要方法：
   - `name()` - 返回工具名称
   - `description()` - 返回工具描述
   - `parameters()` - 定义工具参数结构
   - `execute()` - 实现工具执行逻辑

3. 在 `src/agent/tools/mod.rs` 中导出新工具：

   ```rust
   pub mod my_tool;
   ```

4. 在 `src/agent/tool.rs` 中注册新工具：

   ```rust
   use crate::agent::tools::my_tool::MyTool;
   // ...
   registry.register(MyTool);
   ```

## 工具实现规范

1. 命名规范：
   - 文件名：小写蛇形命名，如 `finish.rs`
   - 工具结构体：大驼峰命名，如 `FinishTool`
   - 工具名称：简明中文描述，如 "完成会话"

2. 参数规范：
   - 使用 JSON Schema 定义参数结构
   - 必须包含 `required` 字段
   - 每个参数应有清晰的描述

3. 执行逻辑：
   - 必须检查 `session.config.tool.{tool_id}.auto_approve`
   - 成功执行后应添加工具调用消息到会话
   - 返回的 `ToolOutput` 应包含执行结果

## 示例参考

```rust
use serde_json::json;
use async_trait::async_trait;
use crate::core::tool::{Tool, ToolError, ToolInput, ToolOutput};
use crate::core::session::SessionData;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "我的工具"
    }

    fn description(&self) -> &str {
        "工具的功能描述"
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "参数1描述"
                }
            },
            "required": ["param1"]
        })
    }

    async fn execute(
        &self, 
        input: ToolInput,
        session: &mut SessionData
    ) -> Result<ToolOutput, ToolError> {
        // 检查自动授权
        if !session.config.tool.base_tool.auto_approve {
            return Err(ToolError::NotAuthorized);
        }

        // 工具逻辑实现...

        // 添加工具消息
        session.add_message(
            crate::core::session::Role::Tool,
            format!("[{}]执行成功", self.name())
        );

        Ok(ToolOutput {
            name: self.name().to_string(),
            result: json!({"status": "success"}),
        })
    }
}
```

## 测试建议

1. 在 `src/agent/tool.rs` 的测试模块中添加工具调用测试
2. 测试应包括：
   - 工具调用解析
   - 参数验证
   - 执行结果验证
