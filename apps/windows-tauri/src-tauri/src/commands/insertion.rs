// paste_test_text — 写剪贴板 + 模拟 Ctrl+V。
//
// macOS 上 enigo 需要 Accessibility 权限；ad-hoc 签的 dev binary 通常拿不到，所以模拟
// 粘贴会静默失败 — 这是 macOS 验收的预期行为。Windows 端验收（Notepad 聚焦后点这个
// 按钮）才是真正的 Phase 0 验收点。
//
// 失败时仍返回 OK + 提示文本「已复制，可手动 Ctrl+V」 — 不阻断主流程，对应产品底线
// 「用户的话不能丢」。

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[tauri::command]
pub fn paste_test_text(text: String) -> Result<String, String> {
    let mut clipboard =
        Clipboard::new().map_err(|err| format!("打开剪贴板失败: {err}"))?;
    clipboard
        .set_text(text.clone())
        .map_err(|err| format!("写剪贴板失败: {err}"))?;

    let paste_result = simulate_paste();

    match paste_result {
        Ok(()) => Ok(format!(
            "clipboard set + Ctrl+V sent (text=\"{}\", len={})",
            preview(&text),
            text.chars().count()
        )),
        Err(err) => Ok(format!(
            "clipboard set ok, 模拟 Ctrl+V 失败（{err}）— 用户可手动粘贴，文本已在剪贴板"
        )),
    }
}

#[cfg(target_os = "macos")]
fn paste_modifier() -> Key {
    Key::Meta
}

#[cfg(not(target_os = "macos"))]
fn paste_modifier() -> Key {
    Key::Control
}

fn simulate_paste() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|err| err.to_string())?;
    let modifier = paste_modifier();
    enigo
        .key(modifier, Direction::Press)
        .map_err(|err| err.to_string())?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|err| err.to_string())?;
    enigo
        .key(modifier, Direction::Release)
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn preview(text: &str) -> String {
    const MAX: usize = 40;
    if text.chars().count() <= MAX {
        text.to_string()
    } else {
        let mut s: String = text.chars().take(MAX).collect();
        s.push('…');
        s
    }
}
