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

## Recent Changes
- web/src/pages/dashboard.tsx
- web/src/components/layout/creator-layout.tsx
- web/src/components/layout/customer-layout.tsx
- web/src/components/layout/app-shell.tsx
- web/src/components/layout/require-role.tsx
- web/src/pages/login.tsx
- web/src/lib/auth-api.ts
- backend/crates/services/src/dto/auth.rs
- backend/crates/services/src/auth_service.rs
- legacy/

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Browser-level three-role routing E2E after roles response change is still pending
- Browser regression uses the installed Chrome channel because the bundled Playwright Chromium crashed in this Windows environment

## Next Step
- Run browser-level three-role routing checks after roles changes; verify admin, creator, and customer login destinations plus forbidden route redirects.
