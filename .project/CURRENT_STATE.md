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
- [x] Payment, settlement, reviews, and full-flow E2E

## Recent Changes
- web/scripts/e2e_flow.py
- web/src/pages/dashboard.tsx
- web/src/lib/reviews-api.ts
- web/src/lib/payment-api.ts
- backend/crates/api/src/routes/appointments.rs
- backend/crates/api/src/routes/reviews.rs
- backend/crates/api/src/routes/payments.rs
- backend/crates/services/src/review_service.rs
- backend/crates/services/src/payment_service.rs
- web/src/lib/admin-api.ts

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents

## Next Step
- Isolate smoke/e2e tests to a test DB; reviews shown on creator profile
