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
- [x] User onboarding system

## Recent Changes
- web/src/pages/service-detail.tsx
- web/src/pages/home.tsx
- web/src/pages/dashboard.tsx
- web/src/pages/account.tsx
- web/src/components/onboarding/empty-state.tsx
- web/src/components/onboarding/step-guide.tsx
- web/scripts/test_runtime.py
- web/scripts/withdrawal_e2e.py
- backend/crates/api/src/routes/mod.rs
- backend/crates/services/tests/withdrawal_flow.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents

## Next Step
- Run browser-level onboarding checks for customer, creator, and admin roles
