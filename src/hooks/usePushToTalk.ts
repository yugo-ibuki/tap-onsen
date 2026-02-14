import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { checkAccessibilityPermission } from "../lib/ipc";

interface UsePushToTalkOptions {
  isRecording: boolean;
  isProcessing: boolean;
  onStart: () => Promise<void>;
  onStop: () => Promise<void>;
}

interface UsePushToTalkReturn {
  isAccessibilityGranted: boolean | null;
  requestAccessibility: () => Promise<void>;
}

export function usePushToTalk({
  isRecording,
  isProcessing,
  onStart,
  onStop,
}: UsePushToTalkOptions): UsePushToTalkReturn {
  const [isAccessibilityGranted, setIsAccessibilityGranted] = useState<
    boolean | null
  >(null);

  // stale closure 対策: 最新の状態を ref で追跡
  const isRecordingRef = useRef(isRecording);
  const isProcessingRef = useRef(isProcessing);
  const onStartRef = useRef(onStart);
  const onStopRef = useRef(onStop);
  // PTT によって開始された録音かどうかを追跡（二重起動防止のガードも兼ねる）
  const pttActiveRef = useRef(false);

  useEffect(() => {
    isRecordingRef.current = isRecording;
  }, [isRecording]);

  useEffect(() => {
    isProcessingRef.current = isProcessing;
  }, [isProcessing]);

  useEffect(() => {
    onStartRef.current = onStart;
  }, [onStart]);

  useEffect(() => {
    onStopRef.current = onStop;
  }, [onStop]);

  // 起動時に Accessibility 権限を確認
  useEffect(() => {
    checkAccessibilityPermission(false).then(setIsAccessibilityGranted);
  }, []);

  // Tauri イベントリスナー（依存なしで1回だけ登録、ref 経由で最新値を参照）
  useEffect(() => {
    let cancelled = false;

    async function setup() {
      const unlistenStart = await listen("ptt-start", () => {
        if (cancelled) return;
        if (
          !isRecordingRef.current &&
          !isProcessingRef.current &&
          !pttActiveRef.current
        ) {
          pttActiveRef.current = true;
          onStartRef.current();
        }
      });

      const unlistenStop = await listen("ptt-stop", () => {
        if (cancelled) return;
        if (pttActiveRef.current) {
          pttActiveRef.current = false;
          onStopRef.current();
        }
      });

      // cleanup が先に呼ばれていた場合に解除
      if (cancelled) {
        unlistenStart();
        unlistenStop();
        return;
      }

      cleanupRef.current = () => {
        unlistenStart();
        unlistenStop();
      };
    }

    const cleanupRef = { current: () => {} };
    setup();

    return () => {
      cancelled = true;
      cleanupRef.current();
    };
  }, []);

  const requestAccessibility = async () => {
    const granted = await checkAccessibilityPermission(true);
    setIsAccessibilityGranted(granted);
  };

  return {
    isAccessibilityGranted,
    requestAccessibility,
  };
}
