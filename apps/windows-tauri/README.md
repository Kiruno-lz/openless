# OpenLess Windows Spike (Phase 0)

Tauri 2 + React + TypeScript + Rust 技术验证。**这一阶段不是产品**，只是用来证明
Windows 端把 OpenLess 主链路跑通所需的几项底层能力是否可行。

> 全文背景见仓库根目录（仅本地保留、未发布到云端）：
>
> - `docs/plans/cross-platform-frontend-adaptation.md`
> - `docs/plans/windows-version-planning.md`

## Phase 0 目标

证明这 6 件事各自可行，6 个按钮一一对应：

| 按钮 | Rust command | 验证什么 |
|---|---|---|
| Register Hotkey | `register_hotkey` | Tauri global-shortcut 能解析 accelerator、监听 keydown |
| Start Mic | `start_microphone_probe` | cpal 能拿到默认输入设备并持续推 PCM |
| Stop Mic | `stop_microphone_probe` | 能把 mic 停掉并返回累计 chunk / level / 时长统计 |
| Paste Test Text | `paste_test_text` | 写剪贴板 + 模拟 Ctrl+V（macOS 是 ⌘+V） |
| Write History | `write_history_probe` | 能把一条 entry 落到 `app_data_dir` 下的 JSON |
| Read History | `read_history_probe` | 重启后还能读回 |

## 不做的事

- 不接 Volcengine ASR / Ark polish（Slice 1 才做）
- 不做完整 UI（Slice 2 才做）
- 不做 installer / Tauri updater（Slice 3 才做）
- 不复刻 macOS SwiftUI 界面

## 本地开发

需要 Node 22+、pnpm 10+、Rust 1.77+。

```bash
cd apps/windows-tauri
pnpm install
pnpm tauri dev      # 开发模式，自动监听 src/ 和 src-tauri/ 重编译
```

`pnpm tauri dev` 会做三件事：

1. 起 Vite dev server（`localhost:1420`）
2. `cargo run` 编译并运行 Rust 主进程
3. Rust 进程打开一个 webview 窗口，加载 spike panel；同时挂上 system tray
   （含 OpenLess + Quit）

### macOS 上能跑，但有限制

| 能力 | macOS 行为 |
|---|---|
| Spike 窗口 | ✅ 能开 |
| Tray 菜单 | ✅ 能挂在 menu bar 上 |
| `Register Hotkey` | ⚠️ 需要 Accessibility 权限；ad-hoc dev binary 通常拿不到，注册可能失败 |
| `Start/Stop Mic` | ⚠️ 第一次点会触发 macOS 麦克风权限弹窗 |
| `Paste Test Text` | ⚠️ 剪贴板部分 OK；模拟 ⌘+V 同样需要 Accessibility，多半会静默失败（错误信息会在日志里） |
| `Write/Read History` | ✅ 完全可用 |

**真正的 Phase 0 验收必须在 Windows 上做。** macOS 只能确认代码骨架没崩。

## Windows 验收清单

按 `docs/plans/windows-version-planning.md` 第 3.1 + 第 6 节，在 Windows 机器上：

```
☐ pnpm tauri dev 能开窗口
☐ system tray 出现 OpenLess 图标，菜单含 OpenLess + Quit
☐ 点 Quit 能退出整个 app
☐ 点 OpenLess（窗口关后）能再调出来
☐ Register Hotkey 注册 Ctrl+Alt+Space 成功，按下后 stderr 有 "[hotkey] fired"
☐ Start Mic 后日志显示采样率 / 通道数 / 设备名（默认麦克风）
☐ Stop Mic 返回 totalChunks > 0、totalBytes > 0、durationMs ≈ 实际时长
☐ Notepad 聚焦后点 Paste Test Text，光标处出现 "OpenLess test"
☐ Edge / Chrome 的 textarea 聚焦后点 Paste Test Text，文字进去
☐ VS Code editor 聚焦后点 Paste Test Text，文字进去
☐ Write History 返回新 entry id；Read History 能读回
☐ 关掉 app、重开、再 Read History，旧 entry 还在
```

把任何「失败」「行为异常」「权限被拒」的项目记下来反馈，决定下一步要不要加 keyboard hook、UI Automation、或换插入策略。

## 目录结构

```
apps/windows-tauri/
  package.json
  vite.config.ts
  tsconfig.json
  index.html
  src/
    main.tsx              入口
    App.tsx / App.css     外壳
    features/spike/SpikePanel.tsx      6 按钮 + 日志区
    lib/tauri/commands.ts              Rust command 类型 + invoke 包装
  src-tauri/
    Cargo.toml
    tauri.conf.json
    build.rs
    capabilities/default.json          global-shortcut 权限
    icons/                              tauri icon 自动生成
    src/
      main.rs                           只调 lib::run()
      lib.rs                            tray + plugin + invoke_handler 注册
      commands/
        mod.rs
        hotkey.rs                       global-shortcut 注册
        audio.rs                        cpal 输入流 + RMS 统计
        insertion.rs                    arboard + enigo 模拟 Ctrl+V
        storage.rs                      app_data_dir 下的 JSON history
```
