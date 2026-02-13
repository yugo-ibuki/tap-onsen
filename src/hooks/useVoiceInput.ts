import { useState, useCallback, useRef } from "react";
import { startRecording, stopRecording, transcribeAudio } from "../lib/ipc";

interface UseVoiceInputReturn {
  isRecording: boolean;
  duration: number;
  transcript: string;
  interimText: string;
  error: string | null;
  start: () => Promise<void>;
  stop: () => Promise<void>;
  clear: () => void;
}

export function useVoiceInput(): UseVoiceInputReturn {
  const [isRecording, setIsRecording] = useState(false);
  const [duration, setDuration] = useState(0);
  const [transcript, setTranscript] = useState("");
  const [interimText, setInterimText] = useState("");
  const [error, setError] = useState<string | null>(null);

  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const startTimer = useCallback(() => {
    setDuration(0);
    timerRef.current = setInterval(() => {
      setDuration((d) => d + 1);
    }, 1000);
  }, []);

  const stopTimer = useCallback(() => {
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const start = useCallback(async () => {
    setError(null);
    setInterimText("...");
    setIsRecording(true);
    startTimer();

    try {
      await startRecording();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setIsRecording(false);
      stopTimer();
      setInterimText("");
    }
  }, [startTimer, stopTimer]);

  const stop = useCallback(async () => {
    setIsRecording(false);
    stopTimer();
    setInterimText("");

    try {
      const recording = await stopRecording();
      const result = await transcribeAudio(
        recording.audio_data,
        recording.sample_rate,
        recording.channels,
      );
      setTranscript((prev) => (prev ? prev + "\n" : "") + result.text);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [stopTimer]);

  const clear = useCallback(() => {
    setTranscript("");
    setInterimText("");
    setError(null);
    setDuration(0);
  }, []);

  return {
    isRecording,
    duration,
    transcript,
    interimText,
    error,
    start,
    stop,
    clear,
  };
}
