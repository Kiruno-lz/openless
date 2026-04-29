// OpenLess Windows Phase 0 spike — 主进程。
//
// 这里只负责：
//   1. 注册 6 个 probe commands（hotkey / mic / paste / history）
//   2. 装上 system tray，含 OpenLess + Quit 两个菜单项
//   3. 装上 global shortcut 插件
//
// 所有真实业务逻辑（ASR、polish、真正的 hotkey -> mic 流水线）都不在 spike 里。

mod commands;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let openless = MenuItemBuilder::with_id("openless", "OpenLess").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&openless, &quit]).build()?;

            let _tray = TrayIconBuilder::with_id("openless-spike-tray")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "openless" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::hotkey::register_hotkey,
            commands::audio::start_microphone_probe,
            commands::audio::stop_microphone_probe,
            commands::insertion::paste_test_text,
            commands::storage::write_history_probe,
            commands::storage::read_history_probe,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
