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

## Recent Changes
- legacy/
- .gitignore
- web/scripts/regression2_e2e.py
- backend/crates/api/src/routes/mod.rs
- backend/crates/api/src/routes/creators.rs
- backend/crates/api/src/routes/service_routes.rs
- backend/crates/api/src/routes/works.rs
- web/src/pages/home.tsx
- web/src/pages/work-detail.tsx
- web/src/pages/creator-profile.tsx

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Creator identity is still profile-based for onboarding; customer and creator capabilities are not yet a first-class role model
- Browser regression uses the installed Chrome channel because the bundled Playwright Chromium crashed in this Windows environment

## Next Step
- Keep legacy/ ignored as an archive; make the next structural pass by retiring root editor/cache directories only after confirming nothing active depends on them.
