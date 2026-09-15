# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT
- [x] Backend image upload endpoint
- [x] AI assistant chat: backend LLM endpoint + frontend panel

## Recent Changes
- web/src/lib/ai-api.ts
- web/src/components/ai/assistant.tsx
- backend/crates/api/src/state.rs
- backend/crates/api/src/routes/ai.rs
- backend/crates/ai/src/chat.rs
- web/scripts/smoke_ui.py
- web/src/lib/auth-api.ts
- web/src/pages/dashboard.tsx
- web/src/pages/home.tsx
- web/package.json

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain

## Next Step
- Creator profile + Service CRUD + persist uploaded works
