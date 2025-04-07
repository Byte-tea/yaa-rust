# 流程分析

## 用户智能体交互

1. 用户输入命令。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   }
   ```

2. yaa 生成根据系统设置生成初始提示词。

   比如会话数据的设置里指定了使用简体中文，提示词就会加上使用简体中文的要求。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   },
   {
       "role": "system",
       "content": "
           <用户任务>帮我收集本周开源热点信息，生成一个热力图。</用户任务>
           <角色>
           您是 `yaa`，一个智能体。
           </角色>
           <工具调用指南>
           ……
           </工具使用指南>
           <目标>
           ……
           </目标>
           ……
           <环境信息>
           ## 操作系统信息：

           GNU/Linux 5.15.0-1028-oem-standard #28~20.04.1-Ubuntu SMP
           PREEMPT_DYNAMIC Thu Mar 23 09:06:06 UTC 2023

           ## 当前时间：
           
           2025-03-29 13:35:04 (Asia/Shanghai, UTC+8:00)
           </环境信息>
           <自定义指示>
           # 用户自定义指示

           以下附加指示由用户提供，应尽量遵守，但不得违反工具使用指南。

           ## 语言偏好

           您应始终使用“简体中文”（zh-CN）语言回答和思考，除非接下来用户给出不同的指示。
           </自定义指示>
       "
   }
   ```

3. 会话数据传递给大模型。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   },
   {
       "role": "system",
       "content": "
           <用户任务>帮我收集本周开源热点信息，生成一个热力图。</用户任务>
           ……
       "
   },
   {
       "role": "assistant",
       "content": "
           根据用户的指示，我应……
           <思考>
           接下来：
           1. 使用“网页访问”工具，访问 https://github.com/trending?since=weekly 的页面。
           ……
           </思考>
           <网页访问>
           <地址>https://github.com/trending?since=weekly</地址>
           </网页访问>
       "
   }
   ```

4. 如果用户未为此工具设置自动授权，会话数据传回用户进行授权。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   },
   {
       "role": "system",
       "content": "
           <用户任务>帮我收集本周开源热点信息，生成一个热力图。</用户任务>
           ……
       "
   },
   {
       "role": "assistant",
       "content": "
           根据用户的指示，我应……
           <思考>
           接下来：
           1. 使用“网页访问”工具，访问 https://github.com/trending?since=weekly 的页面。
           ……
           </思考>
           <网页访问>
           <地址>https://github.com/trending?since=weekly</地址>
           </网页访问>
       "
   },
   {
       "role": "user",
       "content": "
           <授权>
           <工具>网页访问</工具>
           <反馈></反馈>
           </授权>
       "
   }
   ```

5. 会话数据传递给工具调用器。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   },
   {
       "role": "system",
       "content": "
           <用户任务>帮我收集本周开源热点信息，生成一个热力图。</用户任务>
           ……
       "
   },
   {
       "role": "assistant",
       "content": "
           根据用户的指示，我应……
           <思考>
           接下来：
           1. 使用“网页访问”工具，访问 https://github.com/trending?since=weekly 的页面。
           ……
           </思考>
           <网页访问>
           <地址>https://github.com/trending?since=weekly</地址>
           </网页访问>
       "
   },
   {
       "role": "user",
       "content": "
           <授权>
           <工具>网页访问</工具>
           <反馈></反馈>
           </授权>
       "
   },
   {
       "role": "tool",
       "content": "
           [网页访问]的内容：

           ……
       "
   }
   ```

6. 重复步骤三、四、五，直到大模型认为任务已完成，使用“完成会话”工具。

   会话数据的消息部分：

   ```json
   {
       "role": "user",
       "content": "帮我收集本周开源热点信息，生成一个热力图。"
   },
   {
       "role": "system",
       "content": "
           <用户任务>帮我收集本周开源热点信息，生成一个热力图。</用户任务>
           ……
       "
   },
   {
       "role": "assistant",
       "content": "
           根据用户的指示，我应……
           <思考>
           接下来：
           1. 使用“网页访问”工具，访问 https://github.com/trending?since=weekly 的页面。
           ……
           </思考>
           <网页访问>
           <地址>https://github.com/trending?since=weekly</地址>
           </网页访问>
       "
   },
   {
       "role": "user",
       "content": "
           <授权>
           <工具>网页访问</工具>
           <反馈></反馈>
           </授权>
       "
   },
   {
       "role": "tool",
       "content": "
           [网页访问]的内容：

           ……
       "
   },
   {
       "role": "assistant",
       "content": "
           <思考>
           ……
           任务已完成
           </思考>
           <完成会话>
           <理由>热力图已以 Markdown 内嵌 mermaid 图表的形式返回。</理由>
           </完成会话>
       "
   },
   {
       "role": "tool",
       "content": "
           [完成会话]执行成功。
       "
   }
   ```
