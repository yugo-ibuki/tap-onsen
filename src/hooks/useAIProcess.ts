import { useState, useCallback } from "react";
import { processWithAI } from "../lib/ipc";
import type { Mode } from "../types/mode";

interface UseAIProcessReturn {
  processedText: string;
  isProcessing: boolean;
  error: string | null;
  process: (text: string, mode: Mode) => Promise<void>;
  clear: () => void;
}

export function useAIProcess(): UseAIProcessReturn {
  const [processedText, setProcessedText] = useState("");
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const process = useCallback(async (text: string, mode: Mode) => {
    if (!text.trim()) return;

    if (!mode.ai_enabled) {
      setProcessedText(text);
      return;
    }

    setIsProcessing(true);
    setError(null);

    try {
      const result = await processWithAI(text, mode.id);
      setProcessedText(result.text);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
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
