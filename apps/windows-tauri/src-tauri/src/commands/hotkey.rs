// register_hotkey: 把传入的 accelerator 字符串注册成全局热键，命中后只在 stderr 打日志。
//
// Phase 0 不接到 dictation 流水线 — 只是证明：
//   * Tauri 的 global-shortcut 插件能解析 macOS / Windows accelerator
//   * 能拿到 keydown 事件
//
// 真正的 toggle 状态机在 Slice 1 才接进来。

use std::str::FromStr;
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[tauri::command]
pub fn register_hotkey(app: AppHandle, accelerator: String) -> Result<String, String> {
    let shortcut = Shortcut::from_str(&accelerator)
        .map_err(|err| format!("解析 accelerator 失败: {err}"))?;

    let manager = app.global_shortcut();

    if manager.is_registered(shortcut) {
        return Ok(format!("already registered: {accelerator}"));
    }

    let label = accelerator.clone();
    manager
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                eprintln!("[hotkey] fired: {label}");
            }
        })
        .map_err(|err| format!("注册失败: {err}"))?;

    Ok(format!("registered: {accelerator}"))
}
