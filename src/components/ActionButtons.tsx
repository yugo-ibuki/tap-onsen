import { useState, useCallback } from "react";

interface ActionButtonsProps {
  text: string;
  onClear: () => void;
  disabled?: boolean;
}

export function ActionButtons({
  text,
  onClear,
  disabled = false,
}: ActionButtonsProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    if (!text) return;
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, [text]);

  return (
    <div className="action-buttons">
      <button
        className={`action-button copy ${copied ? "copied" : ""}`}
        onClick={handleCopy}
        disabled={disabled || !text}
      >
        {copied ? "✓ コピー済み" : "コピー"}
      </button>
      <button
        className="action-button clear"
        onClick={onClear}
        disabled={disabled || !text}
      >
        クリア
      </button>
    </div>
  );
}
