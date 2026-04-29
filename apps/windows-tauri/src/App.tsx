import { SpikePanel } from "./features/spike/SpikePanel";

export function App() {
  return (
    <main className="app-shell">
      <header className="app-header">
        <h1>OpenLess Windows Spike</h1>
        <p>
          Phase 0 技术验证面板。每个按钮调用一个 Rust command，把结果打印到日志区。
          所有 Windows 平台行为需要在 Windows 机器上验收。
        </p>
      </header>
      <SpikePanel />
    </main>
  );
}
