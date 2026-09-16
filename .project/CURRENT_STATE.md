# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT
- [x] Backend image upload endpoint
- [x] AI assistant chat: backend LLM endpoint + frontend panel
- [x] Persist uploaded works in DB and show on home
- [x] Design system + ui components + explore/work pages
- [x] Creator profile + Service CRUD + appointments
- [x] Seed realistic creators/services/works with real photos

## Recent Changes
- web/src/pages/home.tsx
- backend/crates/services/src/work_service.rs
- backend/crates/db/src/migrations/m20260916_000004_add_work_category.rs
- backend/crates/services/src/seed.rs
- web/src/pages/creator-profile.tsx
- web/src/pages/service-detail.tsx
- web/src/pages/services.tsx
- web/src/lib/marketplace-api.ts
- backend/crates/api/src/routes/appointments.rs
- backend/crates/api/src/routes/service_routes.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db

## Next Step
- Isolate smoke tests to a test DB; creator avatars + reviews
