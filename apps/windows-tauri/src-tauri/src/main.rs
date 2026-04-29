// 真正的入口在 lib.rs，这样以后可以同时编进 desktop 和 mobile target。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    openless_windows_tauri_lib::run();
}
