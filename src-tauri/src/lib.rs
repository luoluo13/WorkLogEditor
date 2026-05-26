use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_sql::{Migration, MigrationKind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    id: String,
    title: String,
    content: String,
    date: String,
    tags: Vec<String>,
}

pub struct DbState(pub Mutex<Connection>);

fn get_db_path(app_handle: &tauri::AppHandle) -> std::path::PathBuf {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("failed to create app data dir");
    }
    path.push("worklog.db");
    path
}

#[tauri::command]
async fn create_log(app_handle: tauri::AppHandle, log: LogEntry) -> Result<(), String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    
    // Ensure table exists before insert (as a safety measure for release build)
    conn.execute("CREATE TABLE IF NOT EXISTS logs (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        content TEXT NOT NULL,
        date TEXT NOT NULL,
        tags TEXT NOT NULL
    );", []).map_err(|e| e.to_string())?;

    let tags_json = serde_json::to_string(&log.tags).unwrap_or_else(|_| "[]".to_string());
    
    conn.execute(
        "INSERT OR REPLACE INTO logs (id, title, content, date, tags) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![log.id, log.title, log.content, log.date, tags_json],
    )
    .map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn update_log(app_handle: tauri::AppHandle, id: String, title: String, content: String, tags: Vec<String>) -> Result<(), String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "UPDATE logs SET title = ?1, content = ?2, tags = ?3 WHERE id = ?4",
        params![title, content, tags_json, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn delete_log(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM logs WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn list_logs(app_handle: tauri::AppHandle) -> Result<Vec<LogEntry>, String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, content, date, tags FROM logs ORDER BY date DESC")
        .map_err(|e| e.to_string())?;
    
    let log_iter = stmt
        .query_map([], |row| {
            let tags_str: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(LogEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                date: row.get(3)?,
                tags,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in log_iter {
        logs.push(log.map_err(|e| e.to_string())?);
    }

    Ok(logs)
}

#[tauri::command]
async fn get_log_by_date(app_handle: tauri::AppHandle, date: String) -> Result<Option<LogEntry>, String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, title, content, date, tags FROM logs WHERE date = ?1")
        .map_err(|e| e.to_string())?;
    
    let mut log_iter = stmt
        .query_map(params![date], |row| {
            let tags_str: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(LogEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                date: row.get(3)?,
                tags,
            })
        })
        .map_err(|e| e.to_string())?;

    if let Some(log) = log_iter.next() {
        Ok(Some(log.map_err(|e| e.to_string())?))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn search_logs(app_handle: tauri::AppHandle, keyword: String) -> Result<Vec<LogEntry>, String> {
    let path = get_db_path(&app_handle);
    let conn = Connection::open(path).map_err(|e| e.to_string())?;

    let query = format!("%{}%", keyword);
    let mut stmt = conn
        .prepare("SELECT id, title, content, date, tags FROM logs WHERE title LIKE ?1 OR content LIKE ?1 ORDER BY date DESC")
        .map_err(|e| e.to_string())?;
    
    let log_iter = stmt
        .query_map(params![query], |row| {
            let tags_str: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
            Ok(LogEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                date: row.get(3)?,
                tags,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in log_iter {
        logs.push(log.map_err(|e| e.to_string())?);
    }

    Ok(logs)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use std::io::{Read as _, Write as _};

#[tauri::command]
async fn export_log_as_markdown(log: LogEntry, path: String) -> Result<(), String> {
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let content = format!("# {}\n\nDate: {}\nTags: {}\n\n{}", 
        log.title, 
        log.date, 
        log.tags.join(", "), 
        log.content
    );
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn export_logs_as_zip(logs: Vec<LogEntry>, zip_path: String) -> Result<(), String> {
    let file = std::fs::File::create(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);

    for log in logs {
        let filename = format!("{}_{}.md", log.date.replace("/", "-"), log.title.replace("/", "-"));
        zip.start_file(filename, options).map_err(|e| e.to_string())?;
        let content = format!("# {}\n\nDate: {}\nTags: {}\n\n{}", 
            log.title, 
            log.date, 
            log.tags.join(", "), 
            log.content
        );
        zip.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn import_from_markdown(path: String) -> Result<LogEntry, String> {
    let mut file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(|e| e.to_string())?;

    let filename = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Log");

    // Simple parsing: first line as title if it starts with #
    let lines: Vec<&str> = content.lines().collect();
    let title = if lines.get(0).map_or(false, |l| l.starts_with("# ")) {
        lines[0][2..].to_string()
    } else {
        filename.to_string()
    };

    let log = LogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        title,
        content,
        date: chrono::Local::now().format("%Y/%m/%d").to_string(),
        tags: vec!["Imported".to_string()],
    };

    Ok(log)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    id: String,
    r#type: String,
    function: ToolFunction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSchema {
    r#type: String,
    function: ToolFunctionDefinition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolFunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[tauri::command]
async fn save_snapshot(app_handle: tauri::AppHandle, base64_data: String, filename: String) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    use base64::{Engine as _, engine::general_purpose};
    
    // Convert base64 to bytes
    let data = base64_data.split(',').last().ok_or("Invalid base64 data")?;
    let bytes = general_purpose::STANDARD.decode(data).map_err(|e| e.to_string())?;

    let path = app_handle.dialog()
        .file()
        .set_file_name(&filename)
        .add_filter("Image", &["png"])
        .blocking_save_file();

    if let Some(file_path) = path {
        std::fs::write(file_path.as_path().ok_or("Invalid path")?, bytes).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Save cancelled".to_string())
    }
}

#[tauri::command]
fn get_all_tools() -> Vec<ToolSchema> {
    vec![
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "get_current_date".to_string(),
                description: "获取当前日期、时间及周数信息".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "list_logs".to_string(),
                description: "获取所有日志列表（包含标题、日期和标签）".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "get_log_by_date".to_string(),
                description: "获取指定日期的详细日志内容".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "date": { "type": "string", "description": "日期，格式为 YYYY/MM/DD" }
                    },
                    "required": ["date"]
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "get_recent_logs".to_string(),
                description: "获取最近 N 天的日志列表".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "days": { "type": "integer", "description": "天数，默认为 7" }
                    }
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "search_logs".to_string(),
                description: "根据关键词搜索日志标题或内容".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "keyword": { "type": "string", "description": "搜索关键词" }
                    },
                    "required": ["keyword"]
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "list_all_tags".to_string(),
                description: "列出所有已使用的标签及其出现频率".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "create_new_log".to_string(),
                description: "创建一条新的工作日志。注意：此操作需要用户确认。".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string", "description": "日志标题" },
                        "content": { "type": "string", "description": "日志 Markdown 内容" },
                        "tags": { "type": "array", "items": { "type": "string" }, "description": "标签列表" }
                    },
                    "required": ["title", "content"]
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "update_log".to_string(),
                description: "修改已有的工作日志。注意：此操作需要用户确认。".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "日志 ID" },
                        "title": { "type": "string", "description": "新的日志标题" },
                        "content": { "type": "string", "description": "新的日志内容" },
                        "tags": { "type": "array", "items": { "type": "string" }, "description": "新的标签列表" }
                    },
                    "required": ["id"]
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "calculate_work_stats".to_string(),
                description: "统计指定时间范围内的日志情况（如标签分布、记录天数等）".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "start_date": { "type": "string", "description": "开始日期 YYYY/MM/DD" },
                        "end_date": { "type": "string", "description": "结束日期 YYYY/MM/DD" }
                    }
                }),
            },
        },
        ToolSchema {
            r#type: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "create_snapshot".to_string(),
                description: "为当前的日志生成一张精美的手绘风格快照图片。".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": { "type": "string", "description": "日志 ID" }
                    },
                    "required": ["id"]
                }),
            },
        },
    ]
}

#[tauri::command]
async fn execute_tool(app_handle: tauri::AppHandle, name: String, arguments: String) -> Result<serde_json::Value, String> {
    let args: serde_json::Value = serde_json::from_str(&arguments).map_err(|e| e.to_string())?;
    
    match name.as_str() {
        "get_current_date" => {
            let now = chrono::Local::now();
            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "date": now.format("%Y/%m/%d").to_string(),
                    "time": now.format("%H:%M:%S").to_string(),
                    "weekday": now.format("%A").to_string(),
                    "week_number": now.format("%U").to_string()
                }
            }))
        },
        "list_logs" => {
            let logs = list_logs(app_handle).await?;
            Ok(serde_json::json!({ "success": true, "data": logs }))
        },
        "get_log_by_date" => {
            let date = args["date"].as_str().ok_or("Missing date argument")?;
            let log = get_log_by_date(app_handle, date.to_string()).await?;
            Ok(serde_json::json!({ "success": true, "data": log }))
        },
        "get_recent_logs" => {
            let days = args["days"].as_i64().unwrap_or(7);
            let logs = list_logs(app_handle).await?;
            let now = chrono::Local::now().date_naive();
            let recent_logs: Vec<LogEntry> = logs.into_iter().filter(|l| {
                if let Ok(log_date) = chrono::NaiveDate::parse_from_str(&l.date, "%Y/%m/%d") {
                    (now - log_date).num_days() < days
                } else {
                    false
                }
            }).collect();
            Ok(serde_json::json!({ "success": true, "data": recent_logs }))
        },
        "search_logs" => {
            let keyword = args["keyword"].as_str().ok_or("Missing keyword argument")?;
            let logs = search_logs(app_handle, keyword.to_string()).await?;
            Ok(serde_json::json!({ "success": true, "data": logs }))
        },
        "list_all_tags" => {
            let logs = list_logs(app_handle).await?;
            let mut tags_count = std::collections::HashMap::new();
            for log in logs {
                for tag in log.tags {
                    *tags_count.entry(tag).or_insert(0) += 1;
                }
            }
            Ok(serde_json::json!({ "success": true, "data": tags_count }))
        },
        "calculate_work_stats" => {
            let logs = list_logs(app_handle).await?;
            let start_date = args["start_date"].as_str().unwrap_or("0000/01/01");
            let end_date = args["end_date"].as_str().unwrap_or("9999/12/31");
            
            let filtered_logs: Vec<LogEntry> = logs.into_iter().filter(|l| {
                l.date >= start_date.to_string() && l.date <= end_date.to_string()
            }).collect();

            let mut tag_distribution = std::collections::HashMap::new();
            for log in &filtered_logs {
                for tag in &log.tags {
                    *tag_distribution.entry(tag.clone()).or_insert(0) += 1;
                }
            }

            Ok(serde_json::json!({
                "success": true,
                "data": {
                    "total_logs": filtered_logs.len(),
                    "tag_distribution": tag_distribution,
                    "period": format!("{} to {}", start_date, end_date)
                }
            }))
        },
        "create_new_log" => {
            Ok(serde_json::json!({
                "success": true,
                "needs_confirmation": true,
                "action": "create_log",
                "data": args
            }))
        },
        "update_log" => {
            Ok(serde_json::json!({
                "success": true,
                "needs_confirmation": true,
                "action": "update_log",
                "data": args
            }))
        },
        "create_snapshot" => {
            Ok(serde_json::json!({
                "success": true,
                "needs_confirmation": false,
                "action": "create_snapshot",
                "data": args
            }))
        },
        _ => Err(format!("Unknown tool: {}", name))
    }
}

#[tauri::command]
async fn get_ollama_models(base_url: String) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    
    let mut models = Vec::new();
    if let Some(models_array) = json["models"].as_array() {
        for m in models_array {
            if let Some(name) = m["name"].as_str() {
                models.push(name.to_string());
            }
        }
    }
    
    Ok(models)
}

#[tauri::command]
async fn ask_ai_with_tools(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<ChatMessage>,
    tools: Option<Vec<ToolSchema>>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    
    // Logic to handle Ollama's base URL and OpenAI compatible endpoint
    let mut url = base_url.trim_end_matches('/').to_string();
    
    // If it looks like a raw Ollama URL (localhost:11434) and doesn't have /v1 or /api
    if (url.contains("11434") || url.contains("localhost")) && !url.contains("/v1") && !url.contains("/api") {
        url = format!("{}/v1", url);
    }
    
    // Append chat/completions if not present
    if !url.ends_with("/chat/completions") {
        url = format!("{}/chat/completions", url);
    }

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.7,
        "stream": false, // Explicitly disable streaming for single JSON response
    });

    if let Some(t) = tools {
        if !t.is_empty() {
            body["tools"] = serde_json::json!(t);
        }
    }

    let mut request = client.post(url);
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let response = request
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API Error ({}): {}", status, error_text));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Handle OpenAI format: result["choices"][0]["message"]
    if let Some(message) = result.get("choices").and_then(|c| c.get(0)).and_then(|m| m.get("message")) {
        Ok(message.clone())
    } else {
        // Fallback for direct model response if some API returns it differently
        Err(format!("Unexpected response format: {}", result))
    }
}

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_logs_table",
            sql: "CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                date TEXT NOT NULL,
                tags TEXT NOT NULL
            );",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:worklog.db", migrations)
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_log,
            update_log,
            delete_log,
            list_logs,
            get_log_by_date,
            search_logs,
            export_log_as_markdown,
            export_logs_as_zip,
            import_from_markdown,
            get_all_tools,
            execute_tool,
            get_ollama_models,
            ask_ai_with_tools,
            save_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
