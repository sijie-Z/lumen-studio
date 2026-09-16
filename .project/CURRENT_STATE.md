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
- [x] Three role interfaces: admin, creator, customer

## Recent Changes
- web/src/lib/admin-api.ts
- web/src/pages/account.tsx
- web/src/pages/admin.tsx
- backend/crates/api/src/routes/admin.rs
- backend/crates/services/src/admin_service.rs
- web/src/pages/home.tsx
- backend/crates/services/src/work_service.rs
- backend/crates/db/src/migrations/m20260916_000004_add_work_category.rs
- backend/crates/services/src/seed.rs
- web/src/pages/creator-profile.tsx

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db

## Next Step
- Payment module + creator appointment confirm + reviews
