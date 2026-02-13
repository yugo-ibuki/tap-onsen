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
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // フォールバック: 旧来のexecCommandを使用
      const textarea = document.createElement("textarea");
      textarea.value = text;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
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
