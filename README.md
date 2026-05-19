# 智能工作日志编辑器 (WorkLog Editor) 

 一款跨平台桌面应用，用于高效记录每日工作日志，并集成 DeepSeek AI 辅助写作与总结。 

 ## 主要功能 

 -  **Markdown 编辑**：不必多言
 -  **AI 辅助**：目前只接入 DeepSeek API，可润色、总结、翻译 
 -  **全文搜索** + 标签筛选：快速定位历史记录 
 -  **导入/导出**：支持 Markdown 和 ZIP 批量备份 
 -  **轻量快速**：基于 Tauri 构建

 > 未来将支持 OpenAI、Ollama 等更多 AI 供应商，敬请期待。 

 ## 快捷键 

 | 操作 | 快捷键 | 
 |------|--------| 
 | 新建日志 | `Ctrl/Cmd + N` | 
 | 保存当前日志 | `Ctrl/Cmd + S` | 
 | 聚焦搜索框 | `Ctrl/Cmd + Shift + F` | 
 | 打开 AI 命令面板 | `Ctrl/Cmd + K` | 

 ##  数据存储 

 所有日志默认保存在用户目录下的 `.worklog` 文件夹中（如 `C:\Users\你的用户名\.worklog`）。您可以在设置中修改存储路径。 

 ##  贡献与反馈 

 本项目开源，欢迎提交 Issue 和 Pull Request。 采纳会将明细写在这里。

 ##  许可证 

 MIT 
