# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT
- [x] Backend image upload endpoint
- [x] AI assistant chat: backend LLM endpoint + frontend panel
- [x] Persist uploaded works in DB and show on home
- [x] Design system + ui components + explore/work pages
- [x] Creator profile + Service CRUD + appointments

## Recent Changes
- web/src/pages/creator-profile.tsx
- web/src/pages/service-detail.tsx
- web/src/pages/services.tsx
- web/src/lib/marketplace-api.ts
- backend/crates/api/src/routes/appointments.rs
- backend/crates/api/src/routes/service_routes.rs
- backend/crates/api/src/routes/creators.rs
- backend/crates/services/src/appointment_service.rs
- backend/crates/services/src/service_catalog.rs
- backend/crates/services/src/creator_service.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain

## Next Step
- Payment module + creator appointment management UI
