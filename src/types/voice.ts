export interface TranscriptionResult {
  text: string;
  confidence: number;
  is_final: boolean;
  timestamp: number;
}

export interface RecordingResult {
  audio_data: number[];
  sample_rate: number;
  channels: number;
  duration_ms: number;
}

export interface AudioState {
  isRecording: boolean;
  duration: number;
  sampleRate: number;
  channelCount: number;
}
