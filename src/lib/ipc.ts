import { invoke } from "@tauri-apps/api/core";
import type { Mode } from "../types/mode";
import type { TranscriptionResult, RecordingResult } from "../types/voice";
import type { AIResponse } from "../types/ai";

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

export async function pasteToForeground(text: string): Promise<void> {
  return invoke<void>("paste_to_foreground", { text });
}
