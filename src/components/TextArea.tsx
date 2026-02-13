interface TextAreaProps {
  inputText: string;
  outputText: string;
  interimText: string;
  isProcessing: boolean;
}

export function TextArea({
  inputText,
  outputText,
  interimText,
  isProcessing,
}: TextAreaProps) {
  const displayText = outputText || inputText;
  const hasContent = displayText || interimText;

  return (
    <div className="text-area-container">
      <div className="text-area">
        {!hasContent && (
          <div className="text-placeholder">
            録音ボタンを押して音声入力を開始してください
          </div>
        )}
        {interimText && <div className="text-interim">{interimText}</div>}
        {displayText && <div className="text-content">{displayText}</div>}
        {isProcessing && (
          <div className="text-processing">
            <span className="processing-dots">AI処理中</span>
          </div>
        )}
      </div>
      {outputText && inputText && outputText !== inputText && (
        <div className="text-source">
          <details>
            <summary>元のテキスト</summary>
            <div className="text-original">{inputText}</div>
          </details>
        </div>
      )}
    </div>
  );
}
