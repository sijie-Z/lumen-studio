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

## Recent Changes
- web/src/pages/admin.tsx
- web/scripts/onboarding_notification_e2e.py
- web/src/components/layout/site-header.tsx
- web/src/lib/notification-api.ts
- backend/crates/db/src/migrations/m20260928_000007_create_notifications.rs
- backend/crates/api/src/routes/notifications.rs
- backend/crates/services/src/notification_service.rs
- web/src/pages/service-detail.tsx
- web/src/pages/home.tsx
- web/src/pages/dashboard.tsx

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents

## Next Step
- Inspect onboarding-notify screenshots and consider WebSocket real-time notification delivery
