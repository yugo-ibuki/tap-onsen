export interface Entry {
  id: number;
  raw_text: string;
  processed_text: string;
  mode_id: string;
  model: string;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  created_at: string;
}

export interface NewEntry {
  raw_text: string;
  processed_text: string;
  mode_id: string;
  model: string;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
}
