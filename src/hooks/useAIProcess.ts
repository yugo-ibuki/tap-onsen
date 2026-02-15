import { useState, useCallback } from "react";
import { processWithAI, saveEntry } from "../lib/ipc";
import type { Mode } from "../types/mode";

interface UseAIProcessReturn {
  processedText: string;
  isProcessing: boolean;
  error: string | null;
  process: (text: string, mode: Mode) => Promise<string>;
  clear: () => void;
}

export function useAIProcess(): UseAIProcessReturn {
  const [processedText, setProcessedText] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const process = useCallback(async (text: string, mode: Mode): Promise<string> => {
    if (!text.trim()) return "";

    if (!mode.ai_enabled) {
      setProcessedText(text);
      // AI無効モードでも履歴に保存（fire-and-forget）
      saveEntry({
        raw_text: text,
        processed_text: text,
        mode_id: mode.id,
        model: "none",
        prompt_tokens: null,
        completion_tokens: null,
        total_tokens: null,
      }).catch((e) => console.warn("Failed to save entry:", e));
      return text;
    }

    setIsProcessing(true);
    setError(null);

    try {
      const result = await processWithAI(text, mode.id);
      setProcessedText(result.text);
      // AI処理結果を履歴に保存（fire-and-forget）
      saveEntry({
        raw_text: text,
        processed_text: result.text,
        mode_id: mode.id,
        model: result.model,
        prompt_tokens: result.usage?.prompt_tokens ?? null,
        completion_tokens: result.usage?.completion_tokens ?? null,
        total_tokens: result.usage?.total_tokens ?? null,
      }).catch((e) => console.warn("Failed to save entry:", e));
      return result.text;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return "";
    } finally {
      setIsProcessing(false);
    }
  }, []);

  const clear = useCallback(() => {
    setProcessedText("");
    setError(null);
  }, []);

  return { processedText, isProcessing, error, process, clear };
}
