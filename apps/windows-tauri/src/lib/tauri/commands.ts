import { invoke } from "@tauri-apps/api/core";

export interface MicProbeStats {
  device: string;
  sampleRate: number;
  channels: number;
  totalChunks: number;
  totalBytes: number;
  lastChunkBytes: number;
  lastLevel: number;
  durationMs: number;
}

export interface HistoryEntryProbe {
  id: string;
  createdAt: string;
  note: string;
}

// Rust 端 commands 命名跟 windows-version-planning.md / cross-platform-frontend-adaptation.md
// 中第 6 步保持一致。每个命令返回 JSON-friendly 结果，前端只负责展示。
export const Commands = {
  registerHotkey(accelerator: string): Promise<string> {
    return invoke<string>("register_hotkey", { accelerator });
  },
  startMicrophoneProbe(): Promise<string> {
    return invoke<string>("start_microphone_probe");
  },
  stopMicrophoneProbe(): Promise<MicProbeStats> {
    return invoke<MicProbeStats>("stop_microphone_probe");
  },
  pasteTestText(text: string): Promise<string> {
    return invoke<string>("paste_test_text", { text });
  },
  writeHistoryProbe(note: string): Promise<HistoryEntryProbe> {
    return invoke<HistoryEntryProbe>("write_history_probe", { note });
  },
  readHistoryProbe(): Promise<HistoryEntryProbe[]> {
    return invoke<HistoryEntryProbe[]>("read_history_probe");
  },
};
