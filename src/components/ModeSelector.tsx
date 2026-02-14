import { useState, useEffect } from "react";
import { getModes } from "../lib/ipc";
import type { Mode } from "../types/mode";

interface ModeSelectorProps {
  selectedMode: Mode | null;
  onModeChange: (mode: Mode) => void;
  disabled?: boolean;
}

export function ModeSelector({
  selectedMode,
  onModeChange,
  disabled = false,
}: ModeSelectorProps) {
  const [modes, setModes] = useState<Mode[]>([]);

  useEffect(() => {
    async function loadModes() {
      try {
        const result = await getModes();
        setModes(result);
        if (!selectedMode && result.length > 0) {
          onModeChange(result[0]);
        }
      } catch (e) {
        console.error("モード設定の読み込みに失敗しました", e);
      }
    }
    loadModes();
  }, []);

  return (
    <div className="mode-selector">
      <div className="mode-selector-label">モード選択</div>
      <div className="mode-options">
        {modes.map((mode) => (
          <label
            key={mode.id}
            className={`mode-option ${selectedMode?.id === mode.id ? "selected" : ""} ${disabled ? "disabled" : ""}`}
          >
            <input
              type="radio"
              name="mode"
              value={mode.id}
              checked={selectedMode?.id === mode.id}
              onChange={() => onModeChange(mode)}
              disabled={disabled}
            />
            <div className="mode-content">
              <span className="mode-label">{mode.label}</span>
              <span className="mode-description">{mode.description}</span>
            </div>
            {mode.ai_enabled && <span className="mode-ai-badge">AI</span>}
          </label>
        ))}
      </div>
    </div>
  );
}
