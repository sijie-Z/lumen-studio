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
- [x] Creator analytics dashboard: settled income, appointment completion, ratings, status and monthly revenue
- [x] Favorites for works, services, and creators (real toggle, status, count, account collection list)
- [x] Validate favorite targets and serve joined favorite list

## Recent Changes
- web/scripts/favorites_e2e.py
- web/src/pages/service-detail.tsx
- web/src/pages/account.tsx
- web/src/lib/favorites-api.ts
- backend/crates/api/src/routes/mod.rs
- backend/crates/services/tests/favorite_flow.rs
- backend/crates/services/src/favorite_service.rs
- .project

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Browser regression uses the installed Chrome channel because the bundled Playwright Chromium crashed in this Windows environment
- Creator analytics has Rust and build coverage but still needs a populated browser visual check

## Next Step
- Run browser-level visual and interaction checks for the creator analytics dashboard, including empty and populated states
