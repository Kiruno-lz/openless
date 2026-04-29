import { useState } from "react";
import { Commands } from "../../lib/tauri/commands";

type LogRow = { ts: string; level: "ok" | "err" | "info"; text: string };

const DEFAULT_HOTKEY = "Ctrl+Alt+Space";
const DEFAULT_PASTE_TEXT = "OpenLess test";

export function SpikePanel() {
  const [log, setLog] = useState<LogRow[]>([]);
  const [busy, setBusy] = useState<string | null>(null);

  function append(row: LogRow) {
    setLog((prev) => [...prev, row].slice(-200));
  }

  async function run(name: string, action: () => Promise<unknown>) {
    setBusy(name);
    const ts = new Date().toISOString().slice(11, 23);
    append({ ts, level: "info", text: `→ ${name}` });
    try {
      const result = await action();
      append({
        ts: new Date().toISOString().slice(11, 23),
        level: "ok",
        text: `✓ ${name}: ${formatResult(result)}`,
      });
    } catch (err) {
      append({
        ts: new Date().toISOString().slice(11, 23),
        level: "err",
        text: `✗ ${name}: ${formatError(err)}`,
      });
    } finally {
      setBusy(null);
    }
  }

  return (
    <section>
      <div className="spike-grid">
        <button
          disabled={busy !== null}
          onClick={() => run("register_hotkey", () => Commands.registerHotkey(DEFAULT_HOTKEY))}
        >
          Register Hotkey
          <br />
          <small>{DEFAULT_HOTKEY}</small>
        </button>
        <button
          disabled={busy !== null}
          onClick={() => run("start_microphone_probe", () => Commands.startMicrophoneProbe())}
        >
          Start Mic
        </button>
        <button
          disabled={busy !== null}
          onClick={() => run("stop_microphone_probe", () => Commands.stopMicrophoneProbe())}
        >
          Stop Mic
        </button>
        <button
          disabled={busy !== null}
          onClick={() => run("paste_test_text", () => Commands.pasteTestText(DEFAULT_PASTE_TEXT))}
        >
          Paste Test Text
          <br />
          <small>"{DEFAULT_PASTE_TEXT}"</small>
        </button>
        <button
          disabled={busy !== null}
          onClick={() =>
            run("write_history_probe", () =>
              Commands.writeHistoryProbe(`spike entry ${new Date().toISOString()}`)
            )
          }
        >
          Write History
        </button>
        <button
          disabled={busy !== null}
          onClick={() => run("read_history_probe", () => Commands.readHistoryProbe())}
        >
          Read History
        </button>
      </div>

      <div className="spike-log">
        {log.length === 0 ? (
          <div className="row">点击上方按钮开始测试。日志只保留最近 200 行。</div>
        ) : (
          log.map((row, i) => (
            <div key={i} className={`row ${row.level}`}>
              {row.ts} {row.text}
            </div>
          ))
        )}
      </div>
    </section>
  );
}

function formatResult(value: unknown): string {
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function formatError(err: unknown): string {
  if (err instanceof Error) return err.message;
  if (typeof err === "string") return err;
  try {
    return JSON.stringify(err);
  } catch {
    return String(err);
  }
}
