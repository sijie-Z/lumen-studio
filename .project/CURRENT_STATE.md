# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT
- [x] Backend image upload endpoint
- [x] AI assistant chat: backend LLM endpoint + frontend panel
- [x] Persist uploaded works in DB and show on home
- [x] Design system + ui components + explore/work pages

## Recent Changes
- web/src/pages/work-detail.tsx
- web/src/pages/explore.tsx
- web/src/components/ui/card.tsx
- web/src/components/ui/button.tsx
- web/uno.config.ts
- web/src/pages/home.tsx
- web/src/pages/dashboard.tsx
- web/src/lib/works-api.ts
- backend/crates/api/src/routes/works.rs
- backend/crates/services/src/work_service.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain

## Next Step
- Creator profile + Service CRUD + appointments
