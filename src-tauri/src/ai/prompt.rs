use crate::config::modes::ModeConfig;

/// プロンプトテンプレートを展開する
///
/// modes.yaml の ai_prompt に含まれる {input} と {context} プレースホルダーを置換する。
/// ai_prompt が未設定の場合はユーザー入力をそのまま返す。
pub fn render_prompt(mode: &ModeConfig, input: &str, context: Option<&str>) -> String {
    match &mode.ai_prompt {
        Some(template) => {
            let mut result = template.clone();

            // {input} をユーザー入力で置換
            if result.contains("{input}") {
                result = result.replace("{input}", input);
            } else {
                // テンプレートに {input} がない場合は末尾に追加
                result = format!("{}\n\n{}", result, input);
            }

            // {context} を直近の入力履歴で置換
            if let Some(ctx) = context {
                result = result.replace("{context}", ctx);
            } else {
                result = result.replace("{context}", "");
            }

            result
        }
        None => input.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mode(ai_prompt: Option<&str>) -> ModeConfig {
        ModeConfig {
            id: "test".to_string(),
            label: "Test".to_string(),
            description: "Test mode".to_string(),
            ai_enabled: true,
            ai_prompt: ai_prompt.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_render_with_input_placeholder() {
        let mode = make_mode(Some("修正してください: {input}"));
        let result = render_prompt(&mode, "こんにちわ", None);
        assert_eq!(result, "修正してください: こんにちわ");
    }

    #[test]
    fn test_render_without_input_placeholder() {
        let mode = make_mode(Some("以下のテキストを校正してください"));
        let result = render_prompt(&mode, "テスト文", None);
        assert_eq!(result, "以下のテキストを校正してください\n\nテスト文");
    }

    #[test]
    fn test_render_with_context() {
        let mode = make_mode(Some("コンテキスト: {context}\n入力: {input}"));
        let result = render_prompt(&mode, "新しい入力", Some("過去の入力"));
        assert_eq!(result, "コンテキスト: 過去の入力\n入力: 新しい入力");
    }

    #[test]
    fn test_render_no_ai_prompt() {
        let mode = make_mode(None);
        let result = render_prompt(&mode, "そのまま返す", None);
        assert_eq!(result, "そのまま返す");
    }
}
