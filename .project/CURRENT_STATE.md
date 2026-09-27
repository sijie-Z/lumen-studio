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

## Recent Changes
- web/src/main.tsx
- web/src/components/layout/app-shell.tsx
- web/src/components/layout/notification-bell.tsx
- web/src/components/layout/require-auth.tsx
- web/src/components/layout/require-role.tsx
- web/src/components/layout/public-layout.tsx
- web/src/components/layout/customer-layout.tsx
- web/src/components/layout/creator-layout.tsx
- web/src/components/layout/admin-layout.tsx
- web/src/components/ui/state.tsx
- web/src/pages/account.tsx
- web/src/pages/dashboard.tsx
- web/src/pages/admin.tsx
- web/src/pages/service-detail.tsx
- web/src/pages/explore.tsx
- web/src/pages/services.tsx
- web/src/pages/work-detail.tsx
- web/src/pages/creator-profile.tsx

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain
- smoke test still writes to main photography.db
- Main session sandbox exec is broken (CreateProcessWithLogonW 1058); work is being driven via sub-agents
- Creator identity is still profile-based for onboarding; customer and creator capabilities are not yet a first-class role model
- Browser-level verification of the four grouped layouts and role redirects is still pending

## Next Step
- Run browser E2E for public/customer/creator/admin layouts, route guards, notification bell, and booking-to-payment handoff
