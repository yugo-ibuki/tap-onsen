use serde::{Deserialize, Serialize};

use crate::ai::client::create_provider;
use crate::ai::prompt::render_prompt;
use crate::ai::ProviderType;
use crate::config::modes::load_modes;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIResponse {
    pub text: String,
    pub model: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// コマンド用の AIResponse を ai モジュールの型から変換する
fn from_ai_response(resp: crate::ai::AIResponse) -> AIResponse {
    AIResponse {
        text: resp.text,
        model: resp.model,
        usage: resp.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    }
}

/// テキストをAIで処理する
///
/// 指定されたモードに応じてプロンプトを組み立て、AIプロバイダーに送信する。
/// モードの ai_enabled が false の場合はテキストをそのまま返す。
#[tauri::command]
pub async fn process_with_ai(text: String, mode_id: String) -> Result<AIResponse, AppError> {
    // モード設定を取得
    let modes =
        load_modes().map_err(|e| AppError::Config(format!("Failed to load modes: {}", e)))?;
    let mode = modes
        .iter()
        .find(|m| m.id == mode_id)
        .ok_or_else(|| AppError::Config(format!("Mode not found: {}", mode_id)))?;

    // AI無効モードの場合はそのまま返す
    if !mode.ai_enabled {
        return Ok(AIResponse {
            text,
            model: "none".to_string(),
            usage: None,
        });
    }

    // プロンプトを組み立て（コンテキストは今回なし — 将来的にステート管理で対応）
    let prompt = render_prompt(mode, &text, None);

    // AI_PROVIDER 環境変数でプロバイダーを選択（vertexai / openai / anthropic）
    let provider_type = match std::env::var("AI_PROVIDER").as_deref() {
        Ok("vertexai") => ProviderType::VertexAI,
        Ok("openai") => ProviderType::OpenAI,
        Ok("anthropic") => ProviderType::Anthropic,
        Ok(other) => {
            return Err(AppError::Ai(format!(
                "Unknown AI_PROVIDER: '{}'. Use vertexai, openai, or anthropic.",
                other
            )));
        }
        Err(_) => {
            return Err(AppError::Ai(
                "AI_PROVIDER not set. Set to vertexai, openai, or anthropic.".to_string(),
            ));
        }
    };

    let provider =
        create_provider(&provider_type).map_err(|e| AppError::Ai(e.to_string()))?;

    // AI処理を実行
    let response = provider
        .process(&prompt)
        .await
        .map_err(|e| AppError::Ai(e.to_string()))?;

    Ok(from_ai_response(response))
}
