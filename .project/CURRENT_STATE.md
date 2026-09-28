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

## Recent Changes
- backend/crates/services/src/review_service.rs
- backend/crates/services/src/creator_service.rs
- backend/crates/api/src/routes/service_routes.rs
- backend/crates/api/src/routes/works.rs
- backend/crates/services/src/auth_service.rs
- backend/crates/services/src/appointment_service.rs
- backend/crates/services/src/withdrawal_service.rs
- backend/crates/ai/src/chat.rs
- backend/crates/api/src/middleware/auth.rs
- backend/crates/api/src/main.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Browser regression uses the installed Chrome channel because the bundled Playwright Chromium crashed in this Windows environment
- Appointment duration validation truncates seconds, so 120m59s passes for a 120-minute service
- Service creation accepts durations outside the 60-480 minute appointment slot range
- SQLite concurrent withdrawal review returns 500 database-is-locked instead of 409; balance remains correct

## Next Step
- Fix exact duration comparison, enforce service duration bounds, and normalize/retry SQLite lock conflicts; then rerun the adversarial audit and full E2E suite
