# 智能工作日志编辑器 (WorkLog Editor) 

 一款基于 Tauri 2.0、Vue 3 和 TypeScript 构建的跨平台桌面应用，结合 Tailwind CSS 与 Naive UI 打造独特的手绘蜡笔视觉风格，并深度集成 DeepSeek AI 辅助写作。 

 ## 主要功能 

 -  **Markdown 编辑**：不必多言
 -  **AI 辅助**：目前只接入 DeepSeek API，可润色、总结、翻译 
 -  **全文搜索** + 标签筛选：快速定位历史记录 
 -  **导入/导出**：支持 Markdown 和 ZIP 批量备份 
 -  **轻量快速**：基于 Tauri 构建

 > 未来将支持 OpenAI、Ollama 等更多 AI 供应商，敬请期待。 
 ##  快速开始

 ### 环境配置

 在开始开发或构建之前，请确保您的系统已安装以下环境：

 1. **Rust 环境**: 
    - 访问 [Rustup.rs](https://rustup.rs/) 下载并安装 Rust 编译器。
    - Windows 用户需要安装 [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)。
 2. **Node.js**: 建议使用 v18 或更高版本。
 3. **包管理器**: 推荐使用 `npm` 。

 ### 启动开发环境

 ```bash
 # 1. 安装前端依赖
 npm install

 # 2. 启动开发模式 (会自动开启 Tauri 窗口)
 npm run tauri dev
 ```

 ### 构建安装包 (.exe)

 ```bash
 # 构建 Windows 安装程序
 npm run tauri build
 ```
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
