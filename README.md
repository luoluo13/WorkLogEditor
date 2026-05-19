# 智能工作日志编辑器 (Worklog Editor) 
  
 一款跨平台桌面应用，用于高效记录每日工作日志，并集成 DeepSeek AI 辅助写作与总结。 
  
 ## ✨ 特性 
  
 - 📝 **Markdown 编辑**：实时预览，简洁高效 
 - 🌓 **深色/浅色主题**：自动跟随系统或手动切换 
 - 🤖 **AI 辅助**：接入 DeepSeek API，可润色、总结、翻译 
 - 🔍 **全文搜索** + 标签筛选：快速定位历史记录 
 - 📦 **导入/导出**：支持 Markdown 和 ZIP 批量备份 
 - 🚀 **轻量快速**：基于 Tauri 构建，内存占用仅 ~50MB 
  
 ## � 下载与安装 
  
 请前往 `https://github.com/yourusername/worklog-editor/releases`  页下载对应操作系统的安装包： 
  
 - Windows：`.exe` 安装程序 
 - macOS：`.dmg` 文件 
 - Linux：`.AppImage` 或 `.deb` 
  
 ## 🔧 首次使用配置 
  
 ### 1. 获取 DeepSeek API Key 
  
 - 访问 `https://platform.deepseek.com/` ，注册并创建 API Key 
 - 新用户通常有免费额度 
  
 ### 2. 在应用中配置 
  
 - 打开应用，点击右上角 **齿轮图标** 进入设置 
 - 填写： 
   - API Key：`sk-xxxxxxxxxxxxxxxx` 
   - Base URL：`https://api.deepseek.com/v1`（默认） 
   - 模型名称：`deepseek-chat`（默认） 
 - 点击“保存” 
  
 > 未来将支持 OpenAI、Ollama 等更多 AI 提供商，敬请期待。 
  
 ## ⌨️ 快捷键 
  
 | 操作 | 快捷键 | 
 |------|--------| 
 | 新建日志 | `Ctrl/Cmd + N` | 
 | 保存当前日志 | `Ctrl/Cmd + S` | 
 | 聚焦搜索框 | `Ctrl/Cmd + Shift + F` | 
 | 打开 AI 命令面板 | `Ctrl/Cmd + K` | 
  
 ## 📂 数据存储 
  
 所有日志默认保存在用户目录下的 `.worklog` 文件夹中（如 `C:\Users\你的用户名\.worklog`）。您可以在设置中修改存储路径。 
  
 ## 🤝 贡献与反馈 
  
 本项目开源，欢迎提交 Issue 和 Pull Request。 
  
 ## 📜 许可证 
  
 MIT 
