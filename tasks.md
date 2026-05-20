**以下是专门为你准备的高质量提示词**，你可以直接复制粘贴到 Cursor / Windsurf / Claude / DeepSeek 等 AI IDE 中使用。

---

**提示词开始**

你现在是一个资深全栈工程师（Tauri + Rust + Vue 3 + TypeScript），正在帮助我为 **WorkLogEditor** 项目添加完整的 **Tool Use（Function Calling）** 框架。

### 项目背景
- 技术栈：Tauri (Rust Backend) + Vue 3 + TypeScript + Vite
- 已有 AI 聊天功能（`ask_ai` command）
- 核心功能是工作日志记录、查看、搜索、总结
- 日志数据以本地文件或 SQLite 形式存储（由你决定合理实现方式）

### 任务目标
为 AI 助手添加 **Tool Calling** 能力，让 AI 能够主动调用工具获取真实日志数据、进行统计、修改日志等操作，实现从“聊天”到“智能工作助手”的升级。

### 推荐架构（必须严格遵循）
1. **Rust Backend** 负责：
   - 定义所有 Tool 的 Schema（OpenAI 兼容格式）
   - 提供 `get_all_tools()` 接口
   - 提供统一的 `execute_tool(name: String, arguments: String)` 接口
   - 实现每个具体工具的业务逻辑

2. **Frontend (Vue + TS)** 负责：
   - 控制 Tool Calling 的多轮循环（ReAct 风格）
   - 处理 Tool 执行结果并塞回对话历史
   - 可视化显示工具调用过程（推荐）

### 需要实现的工具列表（优先级从高到低，必须全部实现）

**核心日志工具：**
- `get_today_logs()` → 获取今天日志
- `get_log_by_date(date: string)` → 指定日期日志 (YYYY-MM-DD)
- `search_logs(keyword?: string, start_date?: string, end_date?: string, tags?: string[])` → 搜索日志
- `get_recent_logs(days: number)` → 最近 N 天日志
- `list_all_tags()` → 所有标签及使用频率
- `summarize_logs(start_date: string, end_date: string, style?: "daily"|"weekly"|"monthly"|"project")` → 工作总结

**生产力工具：**
- `create_new_log(content: string, title?: string, tags?: string[], duration?: number)`
- `update_log(id: string, content?: string, tags?: string[], duration?: number)`
- `calculate_work_stats(start_date: string, end_date: string)` → 时长、标签分布等统计
- `get_current_date()` → 返回当前日期、周数等时间信息

**要求**：
- 所有工具返回统一格式：`{ success: boolean, data: any, message?: string }`
- `update_log`、`create_new_log` 等修改类操作必须在前端做用户确认（工具可先返回需要确认的信息）

### 具体实现要求

**Rust 端需要提供以下 Tauri Commands：**
- `get_all_tools() -> Vec<ToolSchema>`
- `execute_tool(name: String, arguments: String) -> Result<ToolResult, String>`
- `chat_completion(messages: Vec<Message>, tools: Vec<ToolSchema>, model: String, api_key: String, base_url: String)` （可选增强版）

**Frontend 需要实现：**
- 一个 `chatWithTools(userInput: string)` 主函数，包含完整的 Tool Calling 循环
- 支持并行调用多个工具
- 工具调用过程在聊天界面可视化显示（如“正在搜索日志...”）
- 错误处理和最大循环次数限制（建议10轮）

**Tool Schema 要求**：
- 每个工具的 `description` 必须清晰、具体，明确说明什么时候应该调用
- `parameters` 使用标准的 JSON Schema 格式
- 支持数组、枚举、可选参数

**System Prompt 要求**：
请同时生成一个高质量的 System Prompt，强调：
- 必须调用工具获取真实数据，不要幻觉
- 当前日期感知
- 回答风格：专业、简洁、结构化、积极

### 输出要求
请按以下结构完整输出代码：

1. **Rust 代码**（src-tauri/src/ 下相关文件）
   - Tool Schema 定义
   - 所有工具的具体实现
   - `get_all_tools` 和 `execute_tool` 函数
   - 必要的 Command 注册

2. **TypeScript 类型定义**（建议新建 `types/tool.ts`）

3. **Frontend 核心逻辑**（AIPanel.vue 或单独的 `useAIChat.ts`）
   - `chatWithTools` 主函数
   - Tool 调用过程展示逻辑

4. **System Prompt** 完整文本

5. **使用说明**：如何在现有 `ask_ai` 基础上集成，以及注意事项

请使用现代、干净、类型安全的代码，充分注释关键部分。优先保证稳定性和用户体验。

现在开始生成完整实现。

---

**提示词结束**

---

**使用建议**：
- 如果你的 AI IDE 支持超长上下文（如 Claude 3.5/4 或 Cursor），直接整段粘贴即可。
- 如果一次生成不完整，可以告诉 AI：“继续生成 Rust 部分的代码” 或 “接下来生成 Frontend Tool Loop”。
- 生成后记得让我帮你 Review 代码，我可以指出潜在问题和优化点。

需要我帮你调整或加强这个提示词的某个部分吗？（比如更侧重 Rust 或 Frontend）