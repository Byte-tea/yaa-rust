use chrono::Utc;
use std::env::consts::{ARCH, OS};

pub struct Param {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub r#type: String,
}

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Param>,
}

pub struct PromptGenerator;

impl PromptGenerator {
    fn gen_xml(name: &str, content: &str) -> String {
        format!("<{name}>\n{content}\n</{name}>\n")
    }

    fn gen_title_content(title: &str, content: &str) -> String {
        format!("## {title}\n\n{content}\n\n")
    }

    const RULES_INFO: &'static str = r#"<规则>
- 创建新项目（例如应用程序、网站或任何软件项目）时，请将所有新文件组织到专用项目目录中，除非用户另有指定。请有逻辑地构建项目，并遵循所创建的特定项目类型的最佳实现方式。除非另有说明，否则新项目应该无需额外设置即可轻松运行，例如，大多数项目都可以用 HTML、CSS 和 JavaScript 构建——您可以在浏览器中打开它们。
- 在确定要包含的适当结构和文件时，请务必考虑项目的类型（例如 Python、JavaScript、Web 应用程序）。还要考虑哪些文件可能与完成任务最相关，例如，查看项目的自述文件助于您了解项目的依赖关系，您可以将这些依赖关系合并到您编写的代码中。
- 对代码进行更改时，请始终优先考虑采用代码的上下文。确保您的更改与现有代码库兼容，并且它们遵循项目的编码标准和最佳实现方式。
- 不要要求提供不必要的信息。调用提供的工具可以高效且有效地完成用户的请求。完成任务后，必须调用“完成会话”工具向用户显示结果。用户可以提供反馈，您可以利用这些反馈进行改进并重试。
- 您只能调用“提问”工具向用户提问。仅当您需要其他详细信息来完成任务时调用此工具，并确保用清晰简洁的问题，这将帮助您继续完成任务。当您提出问题时，请根据您的问题为用户提供二到四个建议的答案，这样他们就不需要做太多的输入。建议应该是具体的、可行的，并且与已完成的任务直接相关。它们应按优先级或逻辑顺序排序。不过，如果您可以调用其它可用工具以避免向用户提问，请调用那个工具。例如，如果用户提到的文件可能位于外部目录（如桌面文件夹）中，则应调用“列出文件“工具列出桌面文件夹中的文件，并检查提到的文件是否存在，而不是要求用户提供文件路径。
- MCP 操作一次应只调用一个，与其他工具的调用方式类似。请等待确认成功，然后再继续执行其他作。
- 每次调用工具后，请务必等待工具角色的响应，以确认工具执行是否成功。
</规则>"#;

    const OBJECTIVES_INFO: &'static str = r#"<目标>
请一步一步地完成给定的任务，将其分解为清晰的步骤，实事求是、有条不紊地完成它们。
1. 分析用户的任务并设定明确、可实现的目标来完成它。按逻辑顺序确定这些目标的优先级。
2. 按顺序完成这些目标，根据需要，一次调用一个可用的工具。每个目标都要对应解决问题过程中的一个步骤。您会被告知已完成的工作以及剩余的工作。
3. 首先，分析“环境信息”。然后，考虑提供的哪些工具是完成用户任务的最相关工具。接下来，浏览相关工具的每个必需参数，并确定用户是否直接提供或提供了足够的信息来推断值。在决定是否可以推断参数时，请仔细考虑所有上下文，以查看它是否支持特定值。如果所有必需的参数都存在或可以合理推断，请关闭“思考”标签并继续工具调用用。但是，如果缺少必需参数的某个值，请不要调用该工具（包括调用缺失参数的相关标签），而是调用“提问”工具要求用户提供缺失的参数。如果未提供，请勿询问有关可选参数的更多信息。
4. 完成用户的任务后，您必须调用“完成会话”工具向用户显示任务的结果。
5. 用户可能会提供反馈，您可以利用这些反馈进行改进并重试。但不要进行毫无意义的来回对话，即不要以问题或提供进一步帮助来结束您的回答。
</目标>
"#;

    const TOOL_CALL_RULES: &'static str = r#"<工具调用规则>
0. 实事求是。宁可多收集信息也不要假设不确定的事物成立。
1. 在<思考>标签中，评估已知的信息和继续完成任务所需的信息。
2. 选择该任务最适合的工具，并评估是否需要更多信息以继续，以及哪些可用的工具最有效地收集这些信息。例如，调用“列出文件”工具比用“终端调用”工具运行`ls`命令更有效。最好考虑每个可用的工具，并使调用最适合当前任务步骤的工具。
3. 如果需要完成多个操作，请在每一个消息调用一个工具，迭代完成任务，每个工具的使调用都要根据前一个工具的调用结果进行，不要假设任何工具调用的后果。每一步都必须根据前一步的结果进行。
4. 用每个工具指定的 XML 格式来调用工具。
5. 每次调用工具后，工具角色将对该工具的执行结果进行回应。这些结果将为您提供继续完成任务或做出进一步决策所需的信息。此回应可能包括：
   - 工具是否执行成功，和未成功的原因。
   - 响应命令行实时输出，您可能需要思考或采取行动。
   - 与工具调用相关的任何其他相关信息。
6. 每次调用工具后，请务必等待用户确认后再继续。在未明确确认工具角色结果的情况下，切勿假设工具执行成功。
逐步地继续完成任务至关重要，在每次调用工具后等待工具角色的消息，然后再继续执行任务。由此您可：
1. 确认每个步骤成功后再继续。
2. 立即解决出现的任何问题或错误。
3. 根据新信息或意外结果调整您的方案。
4. 确保每次行动都正确地建立在之前的行动上。
请等待并仔细考虑工具角色和用户在每次调用工具后的反应，并缜密从容地应对，做出明智的决定。此迭代过程有助于确保工作整体的成功和行动准确性。
</工具调用规则>"#;

    pub fn generate(
        task: &str,
        time: &str,
        character: &str,
        tools: &[ToolInfo],
        language: &str,
    ) -> String {
        let user_task = Self::gen_xml("用户任务", task);
        let character_info = Self::gen_xml("角色", character);

        let tools_info = Self::gen_tools_info(tools);
        let tools_section = Self::gen_xml("工具调用指南", &tools_info);

        let env_content = {
            let system_info = format!("{} {}", OS, ARCH);
            format!(
                "{}{}",
                Self::gen_title_content("时间", time),
                Self::gen_title_content("系统信息", &system_info)
            )
        };
        let env_info_section = Self::gen_xml("环境信息", &env_content);

        let custom_info = {
            let language_part = Self::gen_title_content(
                "语言偏好",
                &format!(
                    "您应始终用{}语言回答和思考，除非接下来用户给出不同的指示。",
                    language
                ),
            );
            format!(
                "# 用户自定义指示\n\n以下附加指示由用户提供，应尽量遵守，但不得违反工具调用指南。\n\n{}",
                language_part
            )
        };
        let custom_section = Self::gen_xml("自定义指示", &custom_info);

        let mut content = String::new();
        content.push_str(&user_task);
        content.push_str(&character_info);
        content.push_str(&tools_section);
        content.push_str(Self::RULES_INFO);
        content.push_str(Self::OBJECTIVES_INFO);
        content.push_str(&env_info_section);
        content.push_str(&custom_section);

        content
    }

    fn gen_tools_info(tools: &[ToolInfo]) -> String {
        let mut tools_info = String::new();
        tools_info.push_str("# 工具的调用\n\n您有能力调用一套需用户批准的工具。您可在每一条消息中调用一个工具，并在工具角色回复里收到工具的使执行结果。您应逐步调用工具来完成特指定任务，每此工具的调用都基于前一个次工具调用的结果。\n\n## 工具调用格式\n\n工具的调用采用形似 XML 风格的标签。工具名称被包含在开、闭标签之间，每个参数也类似地被包含在其自己的标签集中。结构如下：\n\n<工具名称>\n<第一个参数的名称>第一个值</第一个参数的名称>\n<第二个参数的名称>第二个值</第二个参数的名称>\n……\n</工具名称>\n\n举个例子：\n\n<完成会话>\n<理由>（对任务完成的总结）</理由>\n</完成会话>\n\n请始终遵循此格式调用工具以确保其被正确解析和执行。\n接下来为可用工具信息。\n\n");

        for tool in tools {
            tools_info.push_str(&Self::gen_title_content(&tool.name, &tool.description));

            if !tool.parameters.is_empty() {
                tools_info.push_str("### 参数\n\n");
                for param in &tool.parameters {
                    let required = if param.required {
                        "(必填)"
                    } else {
                        "(可选)"
                    };
                    tools_info.push_str(&format!(
                        "- {}{}，{}：{}\n",
                        param.name, required, param.r#type, param.description
                    ));
                }
                tools_info.push_str("\n");
            }

            tools_info.push_str("### 调用方法\n\n");
            tools_info.push_str(&format!("<{}>\n", tool.name));
            for param in &tool.parameters {
                tools_info.push_str(&format!(
                    "<{}>（{}）</{}>\n",
                    param.name, param.description, param.name
                ));
            }
            tools_info.push_str(&format!("</{}>\n\n", tool.name));
        }

        tools_info.push_str(Self::TOOL_CALL_RULES);
        tools_info
    }

    pub fn generate_with_current_time(
        task: &str,
        character: &str,
        tools: &[ToolInfo],
        language: &str,
    ) -> String {
        let now = Utc::now().to_rfc3339();
        Self::generate(task, &now, character, tools, language)
    }
}

// TODO 测试