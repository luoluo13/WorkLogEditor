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

pub struct DbState(Mutex<Connection>);

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

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    role: String,
    content: String,
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

#[tauri::command]
async fn ask_ai(
    api_key: String,
    base_url: String,
    model: String,
    messages: Vec<ChatMessage>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = if base_url.ends_with("/chat/completions") {
        base_url
    } else {
        format!("{}/chat/completions", base_url.trim_end_matches('/'))
    };

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.7,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API Error ({}): {}", status, error_text));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    let answer = result["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Failed to parse AI response")?;

    Ok(answer.to_string())
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
            ask_ai,
            create_log,
            update_log,
            delete_log,
            list_logs,
            get_log_by_date,
            search_logs,
            export_log_as_markdown,
            export_logs_as_zip,
            import_from_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
