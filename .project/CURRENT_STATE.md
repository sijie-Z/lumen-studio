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
- [x] Message notification system
- [x] Onboarding and notification browser E2E
- [x] Frontend route groups, shared AppShell, and unified route guards
- [x] Core booking-to-payment handoff and four-state UI feedback
- [x] Browser regression for guarded layouts, full transaction, notifications, and cancellation refunds
- [x] Polish creator, work detail, and home pages
- [x] Paginate list APIs and connect the existing PaginatedResponse contract
- [x] Browser regression for pagination and full flow
- [x] Archive legacy code under legacy/
- [x] Introduce first-class customer/creator capabilities without blocking creator onboarding
- [x] Address adversarial security and consistency findings
- [x] Close remaining validation and concurrency gaps
- [x] Address P2 findings: stats, pagination, hardening
- [x] Rebrand to Lumen Studio
- [x] Finish Lumen Studio placeholder rebrand
- [x] Verify Lumen Studio rebrand with four isolated browser regressions
- [x] Stabilize browser regressions by serving the production Vite build

## Recent Changes
- backend/crates/ai/src/chat.rs
- web/src/pages/service-detail.tsx
- web/scripts/test_runtime.py
- web/scripts/regression2_e2e.py
- web/scripts/regression3_e2e.py
- web/scripts/onboarding_notification_e2e.py
- web/scripts/withdrawal_e2e.py

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Long browser flows should use the production preview test runtime because the Vite dev server is resource-heavy on this Windows host

## Next Step
- Continue product-level UI and workflow polish while preserving the verified Lumen Studio baseline
