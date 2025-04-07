//! 工具调用相关功能

use crate::agent::tools::finish::FinishTool;
use crate::agent::tools::rethink::RethinkTool;
use crate::core::tool::ToolRegistry;
use serde_json;

use crate::core::tool::ToolParam;
use std::collections::HashMap;

/// 从字符串中提取工具调用信息
///
/// 从XML格式字符串中提取工具名、参数定义和参数值
pub fn extract_tool_call(
    input: &str,
) -> Option<(&str, Vec<ToolParam>, HashMap<String, serde_json::Value>)> {
    let mut end = input.len();

    while let Some(start) = input[..end].rfind("</") {
        if let Some(tag_end) = input[start..].find('>') {
            let tag_end = start + tag_end;
            let tag_name_start = start + 2;
            let tag_name = &input[tag_name_start..tag_end];

            // 检查是否有对应的开始标签
            if let Some(open_tag_start) = input[..start].rfind(&format!("<{}>", tag_name)) {
                let content_start = open_tag_start + tag_name.len() + 2; // +2 for <>
                let content = &input[content_start..start].trim();

                // 将 XML 参数转换为 ToolParam 列表和参数值映射
                let mut params = Vec::new();
                let mut param_values = HashMap::new();
                let mut param_start = 0;
                while let Some(tag_start) = content[param_start..].find('<') {
                    let tag_start = param_start + tag_start;
                    if let Some(tag_end) = content[tag_start..].find('>') {
                        let tag_end = tag_start + tag_end;
                        let tag_name_start = tag_start + 1;
                        let param_name = &content[tag_name_start..tag_end];

                        if let Some(close_tag) =
                            content[tag_end + 1..].find(&format!("</{}>", param_name))
                        {
                            let close_tag = tag_end + 1 + close_tag;
                            let param_value = &content[tag_end + 1..close_tag].trim();

                            params.push(ToolParam {
                                name: param_name.to_string(),
                                description: String::new(), // 需要从工具定义中获取
                                required: true,             // 默认为必需参数
                                r#type: "string".to_string(), // 默认为字符串类型
                            });

                            param_values.insert(
                                param_name.to_string(),
                                serde_json::Value::String(param_value.to_string()),
                            );

                            param_start = close_tag + param_name.len() + 3; // +3 for </>
                        }
                    }
                }

                return Some((tag_name, params, param_values));
            }
        }
        end = start;
    }

    None
}

/// 创建并注册所有可用工具
pub fn create_tool_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(RethinkTool);
    registry.register(FinishTool);
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_tool_call() {
        let input = r#"一些前置内容
<第一个函数>
<参数>参数内容</参数>
</第一个函数>
<网页访问>
<地址>https://example.com</地址>
<地址二>https://example.com/2</地址二>
</网页访问>"#;

        let result = extract_tool_call(input);
        assert!(result.is_some());
        let (tool_name, params, param_values) = result.unwrap();
        assert_eq!(tool_name, "网页访问");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "地址");
        assert_eq!(params[1].name, "地址二");
        assert_eq!(
            param_values.get("地址"),
            Some(&serde_json::Value::String(
                "https://example.com".to_string()
            ))
        );
        assert_eq!(
            param_values.get("地址二"),
            Some(&serde_json::Value::String(
                "https://example.com/2".to_string()
            ))
        );

        let input = "没有工具调用的内容";
        assert!(extract_tool_call(input).is_none());
    }
}
