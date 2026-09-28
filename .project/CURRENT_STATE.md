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
- [x] Third-round adversarial re-review: exact duration, duration bounds, and concurrent withdrawal review approved
- [x] Add root README with architecture, setup, seed accounts, tests, and known limitations

## Recent Changes
- README.md
- web/scripts/audit_probe.py
- web/scripts/test_runtime.py
- backend/crates/api/src/routes/mod.rs
- backend/crates/services/tests/withdrawal_flow.rs
- backend/crates/common/src/error.rs
- backend/crates/services/src/service_catalog.rs
- backend/crates/services/src/review_service.rs
- backend/crates/services/src/creator_service.rs
- backend/crates/api/src/routes/service_routes.rs
- backend/crates/api/src/routes/works.rs
- backend/crates/services/src/auth_service.rs
- backend/crates/services/src/appointment_service.rs

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Browser-level three-role routing E2E after roles response change is still pending
- Browser regression uses the installed Chrome channel because the bundled Playwright Chromium crashed in this Windows environment
- Postgres-specific row locking remains statically reviewed but not executed against a live Postgres instance

## Next Step
- Publish the repository to GitHub after official GitHub CLI authentication is available.
- Add a Postgres integration test for concurrent withdrawal review when a test database is available.
