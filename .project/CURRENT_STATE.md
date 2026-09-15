# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT
- [x] Backend image upload endpoint
- [x] AI assistant chat: backend LLM endpoint + frontend panel
- [x] Persist uploaded works in DB and show on home

## Recent Changes
- web/src/pages/home.tsx
- web/src/pages/dashboard.tsx
- web/src/lib/works-api.ts
- backend/crates/api/src/routes/works.rs
- backend/crates/services/src/work_service.rs
- backend/crates/db/src/migrations/m20260915_000002_create_works.rs
- backend/crates/db/src/entities/work.rs
- web/src/lib/ai-api.ts
- web/src/components/ai/assistant.tsx
- backend/crates/api/src/state.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain

## Next Step
- Creator profile + Service CRUD + appointments
