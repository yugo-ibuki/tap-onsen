import { useCallback, useEffect, useRef, useState } from "react";
import { ModeSelector } from "./components/ModeSelector";
import { TextArea } from "./components/TextArea";
import { RecordButton } from "./components/RecordButton";
import { ActionButtons } from "./components/ActionButtons";
import { History } from "./components/History";
import { useVoiceInput } from "./hooks/useVoiceInput";
import { useAIProcess } from "./hooks/useAIProcess";
import { usePushToTalk } from "./hooks/usePushToTalk";
import { pasteToForeground } from "./lib/ipc";
import type { Mode } from "./types/mode";
import "./App.css";

function App() {
  const [selectedMode, setSelectedMode] = useState<Mode | null>(null);
  const [historyKey, setHistoryKey] = useState(0);
  const voice = useVoiceInput();
  const ai = useAIProcess();
  const pttTriggeredRef = useRef(false);

  const handlePttStop = useCallback(async () => {
    pttTriggeredRef.current = true;
    await voice.stop();
  }, [voice]);

  const ptt = usePushToTalk({
    isRecording: voice.isRecording,
    isProcessing: ai.isProcessing,
    onStart: voice.start,
    onStop: handlePttStop,
  });

  // 音声認識結果が更新されたらAI処理を実行
  // PTTトリガー時は処理結果を前面アプリにペースト
  useEffect(() => {
    if (voice.transcript && selectedMode) {
      const wasPttTriggered = pttTriggeredRef.current;
      pttTriggeredRef.current = false;

      ai.process(voice.transcript, selectedMode).then((resultText) => {
        setHistoryKey((k) => k + 1);
        if (wasPttTriggered && resultText) {
          pasteToForeground(resultText).catch((e) => {
            console.error("Failed to paste to foreground:", e);
          });
        }
      });
    }
  }, [voice.transcript, selectedMode]);

  const handleClear = useCallback(() => {
    voice.clear();
    ai.clear();
  }, [voice, ai]);

  const displayText = ai.processedText || voice.transcript;

  return (
    <div className="app">
      <header className="app-header">
        <h1 className="app-title">Voice Input App</h1>
      </header>

      <main className="app-main">
        <ModeSelector
          selectedMode={selectedMode}
          onModeChange={setSelectedMode}
          disabled={voice.isRecording}
        />

        <TextArea
          inputText={voice.transcript}
          outputText={ai.processedText}
          interimText={voice.interimText}
          isProcessing={ai.isProcessing}
        />

        <RecordButton
          isRecording={voice.isRecording}
          duration={voice.duration}
          onStart={voice.start}
          onStop={voice.stop}
          disabled={ai.isProcessing}
        />

        {ptt.isAccessibilityGranted === false ? (
          <button
            type="button"
            className="ptt-hint ptt-hint--warning"
            onClick={ptt.requestAccessibility}
          >
            右 ⌥ Option 長押しで録音するにはアクセシビリティ権限が必要です
          </button>
        ) : (
          <p className="ptt-hint">右 ⌥ Option 長押しでも録音できます</p>
        )}

        <ActionButtons
          text={displayText}
          onClear={handleClear}
          disabled={voice.isRecording || ai.isProcessing}
        />

        <History refreshKey={historyKey} />
      </main>

      {(voice.error || ai.error) && (
        <div className="app-error">{voice.error || ai.error}</div>
      )}
    </div>
  );
}

export default App;
