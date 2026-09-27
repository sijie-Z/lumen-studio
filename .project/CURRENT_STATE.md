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
- [x] Creator withdrawal application and admin review
- [x] Withdrawal browser E2E and release commit

## Recent Changes
- web/scripts/test_runtime.py
- web/scripts/withdrawal_e2e.py
- backend/crates/api/src/routes/mod.rs
- backend/crates/services/tests/withdrawal_flow.rs
- web/src/pages/admin.tsx
- backend/crates/db/src/migrations/m20260928_000006_create_withdrawals.rs
- backend/crates/api/src/routes/withdrawals.rs
- backend/crates/services/src/withdrawal_service.rs
- web/scripts/e2e_flow.py
- web/src/pages/dashboard.tsx

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents

## Next Step
- Commit verified withdrawal E2E, creator review display, and isolated test runtime changes
