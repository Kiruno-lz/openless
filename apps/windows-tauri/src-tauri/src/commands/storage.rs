// History probe — 把一条 entry 追加到本地 JSON 文件，再能完整读回。
//
// 文件路径取 Tauri 的 app_data_dir（Windows 是 %APPDATA%\com.openless.windows.spike\
// history-probe.json，macOS 是 ~/Library/Application Support/...）。Phase 0 用最简
// 的 Vec<HistoryEntryProbe> + serde_json 整体读写。Slice 1 上限就要换成 SQLite 或
// append-only JSONL，避免大 history 全量读写。

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const HISTORY_FILE: &str = "history-probe.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryProbe {
    pub id: String,
    pub created_at: String,
    pub note: String,
}

#[tauri::command]
pub fn write_history_probe(app: AppHandle, note: String) -> Result<HistoryEntryProbe, String> {
    let path = history_path(&app)?;
    let mut entries = load(&path)?;

    let entry = HistoryEntryProbe {
        id: format!("probe-{}", Utc::now().timestamp_millis()),
        created_at: Utc::now().to_rfc3339(),
        note,
    };
    entries.push(entry.clone());
    save(&path, &entries)?;

    Ok(entry)
}

#[tauri::command]
pub fn read_history_probe(app: AppHandle) -> Result<Vec<HistoryEntryProbe>, String> {
    let path = history_path(&app)?;
    load(&path)
}

fn history_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("拿 app_data_dir 失败: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| format!("创建 app data dir 失败: {err}"))?;
    Ok(dir.join(HISTORY_FILE))
}

fn load(path: &PathBuf) -> Result<Vec<HistoryEntryProbe>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|err| format!("读 history 失败: {err}"))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&raw).map_err(|err| format!("history JSON 解析失败: {err}"))
}

fn save(path: &PathBuf, entries: &[HistoryEntryProbe]) -> Result<(), String> {
    let json = serde_json::to_string_pretty(entries)
        .map_err(|err| format!("history 序列化失败: {err}"))?;
    fs::write(path, json).map_err(|err| format!("写 history 失败: {err}"))
}
