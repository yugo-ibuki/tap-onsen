use std::collections::VecDeque;
use std::sync::Mutex;

/// 直近の入力履歴を保持するコンテキストマネージャ
pub struct ContextManager {
    history: Mutex<VecDeque<String>>,
    max_entries: usize,
}

impl ContextManager {
    /// 新しい ContextManager を作成する
    ///
    /// `max_entries`: 保持する最大履歴数（デフォルト3）
    pub fn new(max_entries: usize) -> Self {
        Self {
            history: Mutex::new(VecDeque::with_capacity(max_entries)),
            max_entries,
        }
    }

    /// 入力テキストを履歴に追加する
    pub fn add_entry(&self, text: &str) {
        let mut history = self.history.lock().unwrap();
        if history.len() >= self.max_entries {
            history.pop_front();
        }
        history.push_back(text.to_string());
    }

    /// 直近の履歴を改行区切りの文字列として取得する
    ///
    /// 履歴が空の場合は None を返す。
    pub fn get_context(&self) -> Option<String> {
        let history = self.history.lock().unwrap();
        if history.is_empty() {
            None
        } else {
            Some(
                history
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| format!("[{}] {}", i + 1, entry))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )
        }
    }

    /// 履歴をクリアする
    pub fn clear(&self) {
        self.history.lock().unwrap().clear();
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_context() {
        let cm = ContextManager::default();
        assert!(cm.get_context().is_none());
    }

    #[test]
    fn test_add_and_get() {
        let cm = ContextManager::new(3);
        cm.add_entry("first");
        cm.add_entry("second");
        let ctx = cm.get_context().unwrap();
        assert!(ctx.contains("[1] first"));
        assert!(ctx.contains("[2] second"));
    }

    #[test]
    fn test_max_entries_eviction() {
        let cm = ContextManager::new(2);
        cm.add_entry("one");
        cm.add_entry("two");
        cm.add_entry("three");
        let ctx = cm.get_context().unwrap();
        assert!(!ctx.contains("one"));
        assert!(ctx.contains("[1] two"));
        assert!(ctx.contains("[2] three"));
    }

    #[test]
    fn test_clear() {
        let cm = ContextManager::default();
        cm.add_entry("data");
        cm.clear();
        assert!(cm.get_context().is_none());
    }
}
