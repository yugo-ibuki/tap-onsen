# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.
## Project Overview

tap-onsen is a fully-functional Tauri-based voice input application for macOS with AI processing capabilities. The app captures audio, transcribes it using OpenAI Whisper, and processes text through three modes: raw, corrected (AI), or summarized (AI).

**Current Status:** Functional frontend and backend with mock data fallbacks. Ready for API integration.

**Tech Stack:** React 19 + TypeScript + Vite frontend, Tauri 2 (Rust) backend, OpenAI Whisper API, macOS CoreAudio

## Project Structure

Frontend: src/ (React components, hooks, types, IPC wrappers)
Backend: src-tauri/ (Rust commands, voice processing, AI integration, config)
Config: config/modes.yaml (three modes: raw, correct, summarize)

## Common Development Commands

pnpm dev        - Start dev server with Tauri window
pnpm build      - Build frontend
pnpm tauri build - Build macOS binary

## Architecture Overview

Data Flow: User Audio → RecordButton → useVoiceInput hook → Tauri IPC → Rust backend (audio capture/Whisper API) → transcript → AI processing → ModeSelector determines if AI applied → output displayed in TextArea

## Key IPC Commands (in src/lib/ipc.ts)

- getModesCommand() - Load modes from config/modes.yaml
- startRecording() - Begin audio capture
- stopRecording() - End audio, return PCM buffer
- transcribeAudio(audio) - Send to Whisper API
- processWithAI(text, modeId) - Apply AI processing

## Frontend Hooks

useVoiceInput - Manages recording state, transcription, with mock data fallback
useAIProcess - Manages AI text processing, handles isProcessing state
Both hooks used in App.tsx, state flows to components for rendering

## Modes Configuration (config/modes.yaml)

raw: no AI processing, direct transcription
correct: AI grammar/spelling correction via prompt template
summarize: AI text summarization via prompt template
Each mode has id, label (Japanese), description, ai_enabled flag, and ai_prompt template

## Completed Implementation

✅ Full React frontend with all UI components
✅ Tauri project setup and Rust scaffolding
✅ Mode configuration system (YAML)
✅ Voice recording state management with mock fallback
✅ Type definitions and IPC wrappers
✅ useVoiceInput and useAIProcess hooks

## TODO - Backend Implementation

🚧 Whisper API client (src-tauri/src/voice/whisper_api.rs)
🚧 AI provider clients: OpenAI & Anthropic (src-tauri/src/ai/)
🚧 Audio capture implementation (CoreAudio integration)
🚧 PCM to WAV format conversion
🚧 Streaming response handling
🚧 API error handling and retry logic
🚧 Environment variable config for API keys

## Environment Variables

OPENAI_API_KEY - Whisper transcription and/or GPT processing
ANTHROPIC_API_KEY - Claude API alternative for AI processing
OPENAI_ORG_ID - Optional OpenAI organization ID
AUDIO_SAMPLE_RATE - Default 16000
AUDIO_CHANNELS - Default 1 (mono)

## Component Architecture

App.tsx: Root component with useVoiceInput and useAIProcess hooks
  ↓ selectedMode state, voice/ai states
  → ModeSelector: Radio buttons for mode selection
  → TextArea: Display interim and final transcription + AI output
  → RecordButton: Start/stop recording with timer
  → ActionButtons: Copy/clear functionality

## Type Definitions Location

Mode interface: src/types/mode.ts
  - id, label, description, ai_enabled, ai_prompt?
TranscriptionResult: src/types/voice.ts
  - text, confidence?, duration?
AIResponse: src/types/ai.ts
  - text, model, tokens? (input/output), error?

## Debugging

Frontend: Right-click in Tauri window → Open DevTools → Console for IPC errors
Rust: cargo check && cargo test in src-tauri/
Mock Mode: useVoiceInput provides MOCK_TRANSCRIPTIONS when API calls fail
Error Display: App.tsx shows voice.error or ai.error in app-error div

## File Organization Reference

Frontend components: src/components/*.tsx (one per component)
Custom hooks: src/hooks/*.ts (useVoiceInput, useAIProcess)
Type definitions: src/types/*.ts (separate files per domain)
IPC layer: src/lib/ipc.ts (Tauri command wrappers)
Backend commands: src-tauri/src/commands/*.rs
Voice pipeline: src-tauri/src/voice/*.rs
AI module: src-tauri/src/ai/*.rs
Config loader: src-tauri/src/config/*.rs

## Tauri Integration Points

Handlers registered in src-tauri/src/lib.rs:
  - commands::get_modes
  - commands::audio::start_recording
  - commands::audio::stop_recording
  - commands::audio::transcribe_audio
  - commands::ai::process_with_ai
Frontend calls via src/lib/ipc.ts wrapper functions

## Dependencies Key Packages

Frontend: React 19, TypeScript 5.7, Vite 6, @tauri-apps/api
Backend: tauri 2, serde/serde_json/serde_yaml, tokio, reqwest, hound, async-trait, futures
