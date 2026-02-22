import { useCallback, useEffect, useState } from "react";
import { getEntries, deleteEntry } from "../lib/ipc";
import type { Entry } from "../types/db";

interface HistoryProps {
  /** 新しいエントリが追加されたときにインクリメントするカウンタ */
  refreshKey: number;
}

export function History({ refreshKey }: HistoryProps) {
  const [entries, setEntries] = useState<Entry[]>([]);
  const [copiedId, setCopiedId] = useState<number | null>(null);

  const load = useCallback(async () => {
    try {
      const data = await getEntries(50, 0);
      setEntries(data);
    } catch (e) {
      console.error("Failed to load history:", e);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load, refreshKey]);

  const handleCopy = useCallback(async (entry: Entry) => {
    const text = entry.processed_text || entry.raw_text;
    await navigator.clipboard.writeText(text);
    setCopiedId(entry.id);
    setTimeout(() => setCopiedId(null), 2000);
  }, []);

  const handleDelete = useCallback(
    async (id: number) => {
      try {
        await deleteEntry(id);
        await load();
      } catch (e) {
        console.error("Failed to delete entry:", e);
      }
    },
    [load],
  );

  if (entries.length === 0) {
    return null;
  }

  return (
    <section className="history">
      <h2 className="history-title">履歴</h2>
      <ul className="history-list">
        {entries.map((entry) => (
          <li key={entry.id} className="history-item">
            <div className="history-item-body">
              <p className="history-text">
                {entry.processed_text || entry.raw_text}
              </p>
              <div className="history-meta">
                <span className="history-mode">{entry.mode_id}</span>
                <span className="history-date">
                  {formatDate(entry.created_at)}
                </span>
              </div>
            </div>
            <div className="history-actions">
              <button
                type="button"
                className={`history-btn history-btn-copy ${copiedId === entry.id ? "copied" : ""}`}
                onClick={() => handleCopy(entry)}
                title="コピー"
              >
                {copiedId === entry.id ? "✓" : "⧉"}
              </button>
              <button
                type="button"
                className="history-btn history-btn-delete"
                onClick={() => handleDelete(entry.id)}
                title="削除"
              >
                ✕
              </button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  const month = d.getMonth() + 1;
  const day = d.getDate();
  const hours = String(d.getHours()).padStart(2, "0");
  const minutes = String(d.getMinutes()).padStart(2, "0");
  return `${month}/${day} ${hours}:${minutes}`;
}
