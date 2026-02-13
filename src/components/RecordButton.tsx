interface RecordButtonProps {
  isRecording: boolean;
  duration: number;
  onStart: () => void;
  onStop: () => void;
  disabled?: boolean;
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${String(s).padStart(2, "0")}`;
}

export function RecordButton({
  isRecording,
  duration,
  onStart,
  onStop,
  disabled = false,
}: RecordButtonProps) {
  return (
    <div className="record-button-container">
      <button
        className={`record-button ${isRecording ? "recording" : ""}`}
        onClick={isRecording ? onStop : onStart}
        disabled={disabled}
        aria-label={isRecording ? "録音停止" : "録音開始"}
      >
        {isRecording ? (
          <>
            <span className="record-icon stop">■</span>
            <span className="record-label">停止</span>
          </>
        ) : (
          <>
            <span className="record-icon mic">🎤</span>
            <span className="record-label">録音開始</span>
          </>
        )}
      </button>
      {isRecording && (
        <div className="record-duration">{formatDuration(duration)}</div>
      )}
    </div>
  );
}
