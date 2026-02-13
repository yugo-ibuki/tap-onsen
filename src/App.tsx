import { useCallback, useEffect } from "react";
import { ModeSelector } from "./components/ModeSelector";
import { TextArea } from "./components/TextArea";
import { RecordButton } from "./components/RecordButton";
import { ActionButtons } from "./components/ActionButtons";
import { useVoiceInput } from "./hooks/useVoiceInput";
import { useAIProcess } from "./hooks/useAIProcess";
import type { Mode } from "./types/mode";
import { useState } from "react";
import "./App.css";

function App() {
  const [selectedMode, setSelectedMode] = useState<Mode | null>(null);
  const voice = useVoiceInput();
  const ai = useAIProcess();

  // 音声認識結果が更新されたらAI処理を実行
  useEffect(() => {
    if (voice.transcript && selectedMode) {
      ai.process(voice.transcript, selectedMode);
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

        <ActionButtons
          text={displayText}
          onClear={handleClear}
          disabled={voice.isRecording || ai.isProcessing}
        />
      </main>

      {(voice.error || ai.error) && (
        <div className="app-error">{voice.error || ai.error}</div>
      )}
    </div>
  );
}

export default App;
