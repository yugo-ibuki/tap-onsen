import { invoke } from "@tauri-apps/api/core";
import type { Mode } from "../types/mode";
import type { TranscriptionResult, RecordingResult } from "../types/voice";
import type { AIResponse } from "../types/ai";
import type { Entry, NewEntry } from "../types/db";

export async function getModes(): Promise<Mode[]> {
  return invoke<Mode[]>("get_modes");
}

export async function transcribeAudio(
  audioData: number[],
  sampleRate: number,
  channels: number,
  engine: "native" | "whisper" = "native",
): Promise<TranscriptionResult> {
  return invoke<TranscriptionResult>("transcribe_audio", {
    audioData,
    sampleRate,
    channels,
    engine,
  });
}

export async function processWithAI(
  text: string,
  modeId: string,
): Promise<AIResponse> {
  return invoke<AIResponse>("process_with_ai", { text, modeId });
}

export async function startRecording(): Promise<void> {
  return invoke<void>("start_recording");
}

export async function stopRecording(): Promise<RecordingResult> {
  return invoke<RecordingResult>("stop_recording");
}

export async function checkAccessibilityPermission(
  prompt: boolean,
): Promise<boolean> {
  return invoke<boolean>("check_accessibility_permission", { prompt });
}

export async function saveEntry(entry: NewEntry): Promise<number> {
  return invoke<number>("save_entry", { entry });
}

export async function getEntries(
  limit: number,
  offset: number,
): Promise<Entry[]> {
  return invoke<Entry[]>("get_entries", { limit, offset });
}

export async function getEntry(id: number): Promise<Entry | null> {
  return invoke<Entry | null>("get_entry", { id });
}

export async function deleteEntry(id: number): Promise<boolean> {
  return invoke<boolean>("delete_entry", { id });
}

export async function pasteToForeground(text: string): Promise<void> {
  return invoke<void>("paste_to_foreground", { text });
}
