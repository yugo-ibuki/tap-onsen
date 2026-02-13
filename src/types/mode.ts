export interface Mode {
  id: string;
  label: string;
  description: string;
  ai_enabled: boolean;
  ai_prompt?: string;
}

export interface AIConfig {
  provider: "openai" | "anthropic" | "local";
  model: string;
  apiKey?: string;
  timeout: number;
  maxRetries: number;
}
