# 任务

1. 了解 `src\` 下所有有必要模块，阅读 `configs/Config.example.json` 和 `Config.json`。
2. 每一步都要用 cargo check 验证无误。
3. 实现参数：`--config （文件名）` 如 `--config ./configs/Config.example.json` 或 `--config ./Config.json` 用于指定默认配置的内容。
4. 根据本任务更新 `Task.md`。

## 当前进展

这是 yaa 智能体后端。它原本是一个 Python 智能体项目，现在需要用 rust 重写它，要支持 wasi（服务器模式、命令行交互模式）和 wasm（能被 HTML 调用 session 模块生成、管理会话数据，能被 HTML 调用 agent 模块进行核心逻辑后取得结果）。

已完成：

1. 实现 SessionData 及相关数据结构
2. 创建工具调用框架
3. 添加依赖项
4. 完善模块导出结构
5. 创建提示词生成器
6. 创建大语言模型 API 调用框架
7. 实现工具调用检查函数（返回工具名和参数内容）
8. 实现"再度思考"工具
9. 实现"完成会话"工具
10. 实现智能体回复数据结构（AgentResponseData 和 StreamResponseData）
11. 实现智能体核心逻辑模块
12. 实现命令行参数解析(--serve和--port参数)

目录：

```bash
src/
├── main.rs              # 入口
├── agent/               # 智能体实现
│   ├── mod.rs           # 智能体模块核心逻辑
│   ├── api.rs
│   ├── tool.rs          # 实现工具调用的具体解析逻辑
│   └── tools/           # 工具实现
│       ├── rethink.rs   # 再度思考工具实现
│       └── finish.rs    # 完成会话工具实现
├── core/                # 核心
│   ├── mod.rs           # 核心初始化
│   ├── session.rs       # 会话数据的结构和智能体响应数据的结构定义
│   ├── tool.rs          # 定义工具框架基础结构和接口
│   └── prompt.rs        # 提示词生成器
└── cli/                 # 命令行模式(部分实现)
    ├── mod.rs           # CLI 入口(待完善)
    └── commands.rs      # 子命令实现(待完善)
```

## 项目核心逻辑

命令行客户端或网页客户端将 session_data 传入 agent 进行处理，agent 会循环执行`提示词生成（包装用户需求，告诉大模型如何调用工具）、大语言模型 API 调用（让大语言模型根据提示词描述的环境和工具调用方法调用一个工具）、工具调用（调用大语言模型指定的工具）`直到大语言模型认为用户的任务已完成，并调用“完成会话”工具。

实现细节：

- 提示词生成器：
  - 如果当前会话数据的最后一条消息的 role 为 user，才使用提示词生成器。
  - 提示词生成器的输入分别取：
    - task：会话数据最后一条消息的 content。
    - time：当前时间（年月日时分秒加时区）。
    - language：从会话数据的配置里取。
  - 生成的提示词，存入 session_data 的 messages 中（role 为 system）。
- 大语言模型 API 调用：
  - 如果当前会话数据的最后一条消息的 role 不为 assistant，才调用大语言模型 API。
  - 创建实例时，从会话数据配置中读取大语言模型 API 的配置。
  - 大模型的回复，存入 session_data 的 messages 中（role 为 assistant）。
- extract_tool_call 返回要调用的工具名称，如果为 None，添加警告“[警告]每次回复至少调用一个工具！”存入 session_data 的 messages 中（role 为 system），并 continue 循环。
- 工具调用：
  - 如果调用的工具存在，且有授权（会话数据的 config 中配置了该工具的自动授权、或最后一条消息的 content 中有`<批准>\n<工具名称>（工具名称）</工具名称>\n</批准>`），则调用工具；否则将会话数据的状态设定为 Interrupted，并 break。
- 检查会话数据的状态是否为 InProgress，如果不是，则跳出循环，返回智能体回复数据。
- 智能体回复数据的 messages 的内容，是智能体处理完的会话数据的 messages 中最后一个 role 为 user 的消息之后的所有消息。

## BUG

1. 当消息角色为 Tool 时调用 API，API 返回的数据无法格式化为 JSON。怀疑其它非 User 和 Assistant 的消息角色也会出错。

## 参考资料

[项目文档](docs/README.md)
[工具调用的实现](docs/FlowAnalyze/FlowAnalyze.md)
