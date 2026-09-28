# Session Log

> Append-only timeline. Never edit past entries.

### [2026-09-15 22:38] main — Auth module: register/login/JWT

- **Task**: Auth module: register/login/JWT
- **Branch**: main
- **Summary**: Auth module complete: SQLite migration, register, login, JWT, protected /me, 11 tests
- **Next**: Creator profile + Service CRUD module

### [2026-09-15 23:30] main — Backend image upload endpoint

- **Task**: Backend image upload endpoint
- **Branch**: main
- **Summary**: Backend image upload endpoint: multipart, auth, 15MB, local storage
- **Next**: Complete: Backend image upload endpoint

### [2026-09-15 23:35] main — AI assistant chat: backend LLM endpoint + frontend panel

- **Task**: AI assistant chat: backend LLM endpoint + frontend panel
- **Branch**: main
- **Summary**: AI assistant chat: backend LLM endpoint + frontend panel
- **Next**: Complete: AI assistant chat: backend LLM endpoint + frontend panel

### [2026-09-15 23:47] main — Persist uploaded works in DB and show on home

- **Task**: Persist uploaded works in DB and show on home
- **Branch**: main
- **Summary**: Persist uploaded works in DB and show on home
- **Next**: Complete: Persist uploaded works in DB and show on home

### [2026-09-16 16:42] main — Design system + ui components + explore/work pages

- **Task**: Design system + ui components + explore/work pages
- **Branch**: main
- **Summary**: Design system + ui components + explore/work pages
- **Next**: Complete: Design system + ui components + explore/work pages

### [2026-09-16 17:29] main — Creator profile + Service CRUD + appointments

- **Task**: Creator profile + Service CRUD + appointments
- **Branch**: main
- **Summary**: Creator profile + Service CRUD + appointments
- **Next**: Complete: Creator profile + Service CRUD + appointments

### [2026-09-16 21:32] main — Seed realistic creators/services/works with real photos

- **Task**: Seed realistic creators/services/works with real photos
- **Branch**: main
- **Summary**: Seed realistic creators/services/works with real photos
- **Next**: Complete: Seed realistic creators/services/works with real photos

### [2026-09-17 07:16] main — Three role interfaces: admin, creator, customer

- **Task**: Three role interfaces: admin, creator, customer
- **Branch**: main
- **Summary**: Three role interfaces: admin, creator, customer
- **Next**: Complete: Three role interfaces: admin, creator, customer

### [2026-09-27 15:45] main — Payment, settlement, reviews, and full-flow E2E

- **Task**: Payment, settlement, reviews, and full-flow E2E
- **Branch**: main
- **Summary**: Payment, settlement, reviews, full-flow E2E
- **Next**: Complete: Payment, settlement, reviews, and full-flow E2E

### [2026-09-27 16:23] main — Creator withdrawal application and admin review

- **Task**: Creator withdrawal application and admin review
- **Branch**: main
- **Summary**: Implemented creator withdrawal application, balance freeze, admin approval/rejection with refund, API routes, withdrawal UI, UserDto balance, and service/API tests. cargo check -p api, cargo test -p services, cargo test -p api, and pnpm build all passed.
- **Next**: Run browser E2E for creator withdrawal apply and admin approve/reject against a logged-in session

### [2026-09-27 16:31] main — Withdrawal browser E2E and release commit

- **Task**: Withdrawal browser E2E and release commit
- **Branch**: main
- **Summary**: Added isolated browser E2E for withdrawal apply/approve/reject with SHA-256 main DB isolation checks. All four verification commands passed. Ready to commit .project, backend/Cargo.lock, backend/crates, and web.
- **Next**: Commit verified withdrawal E2E, creator review display, and isolated test runtime changes

### [2026-09-27 16:39] main — User onboarding system

- **Task**: User onboarding system
- **Branch**: main
- **Summary**: Implemented reusable StepGuide and EmptyState, then added role-based onboarding to customer, creator, admin, home, and service detail pages. pnpm build passed.
- **Next**: Run browser-level onboarding checks for customer, creator, and admin roles

### [2026-09-27 16:40] main — Message notification system

- **Task**: Message notification system
- **Branch**: main
- **Summary**: Implemented notifications table/entity/migration, NotificationService, authenticated list/unread/read/read-all APIs, event triggers for appointment/payment/completion/withdrawal, and a polling notification bell in the shared header. cargo check -p api, cargo test -p api (7 tests), and pnpm build passed.
- **Next**: Browser E2E: appointment/payment/completion/withdrawal notifications and unread badge

### [2026-09-27 16:48] main — Onboarding and notification browser E2E

- **Task**: Onboarding and notification browser E2E
- **Branch**: main
- **Summary**: Added isolated Playwright E2E for onboarding and notifications. Fixed missing notification bell on account/dashboard/admin role headers. Verified full customer/creator/admin flow, notification badges and read-all, onboarding dismissal persistence, 14 screenshots, port cleanup, and unchanged main DB SHA-256.
- **Next**: Inspect onboarding-notify screenshots and consider WebSocket real-time notification delivery

### [2026-09-27 20:23] main — Frontend structure refactor and core flow completion

- **Task**: Grouped layouts, unified shell, route guards, IA cleanup, and booking-to-payment handoff
- **Branch**: main
- **Files**: web/src/main.tsx; web/src/components/layout/app-shell.tsx; web/src/components/layout/notification-bell.tsx; web/src/components/layout/require-auth.tsx; web/src/components/layout/require-role.tsx; web/src/components/layout/public-layout.tsx; web/src/components/layout/customer-layout.tsx; web/src/components/layout/creator-layout.tsx; web/src/components/layout/admin-layout.tsx; web/src/components/ui/state.tsx; web/src/pages/account.tsx; web/src/pages/dashboard.tsx; web/src/pages/admin.tsx; web/src/pages/service-detail.tsx; web/src/pages/explore.tsx; web/src/pages/services.tsx; web/src/pages/work-detail.tsx; web/src/pages/creator-profile.tsx
- **Decision**: ADR-017 grouped routes by public/customer/creator/admin, moved navigation and session guards to layouts, removed customer appointments from creator workspace, and kept creator onboarding enabled for authenticated customers
- **Verification**: pnpm build passed with tsc + Vite; 1845 modules transformed
- **Risks**: Browser-level route guard and responsive shell verification is still pending; creator capability remains profile-based during onboarding
- **Next**: Run grouped-layout browser E2E for public, customer, creator, and admin flows

### [2026-09-27 20:49] main — Full browser regression for guarded layouts and refunds

- **Task**: Full browser-level regression for route guards, unified shell, transaction flow, cancellation refund, payment handoff, onboarding, and notifications
- **Branch**: main
- **Files**: web/scripts/regression_e2e.py; web/src/pages/account.tsx; web/index.html; web/public/favicon.svg
- **Summary**: Added isolated A-F Playwright regression using test_runtime.py, Chrome channel, 14 screenshots, API balance/payment assertions, and main DB SHA-256 comparison. Fixed the customer onboarding guide being hidden for users with zero appointments, and added a favicon to remove the final browser 404.
- **Verification**: Regression E2E passed A-F; cargo test -p services --test transaction_consistency passed 7/7; pnpm build passed; main photography.db SHA-256 unchanged; ports 8080 and 5173 released.
- **Risks**: Chrome channel is required on this machine because bundled Playwright Chromium crashes in the current Windows environment.
- **Next**: Extend browser coverage to pagination, search/filtering, creator onboarding, and refund notification semantics

### [2026-09-27 21:43] main — Polish creator, work detail, and home pages

- **Task**: Polish creator, work detail, and home pages
- **Branch**: main
- **Summary**: Polished creator profile with real work filtering, tabs, identity header and four-state UI; rebuilt work detail with real creator routing, removed fake favorite/contact actions and added related works; reworked home hierarchy, query states and responsive spacing. pnpm build passed with 1846 modules; only web pages were changed.
- **Next**: Run browser-level visual checks for creator profile, work detail, home, and then extend search/filter coverage

### [2026-09-28 00:58] main — Paginate list APIs and connect the existing PaginatedResponse contract

- **Task**: Paginate list APIs and connect the existing PaginatedResponse contract
- **Branch**: main
- **Summary**: Implemented paginated works, services, and creators APIs with search/category/type/location filters; connected explore and services pages to server-side pagination and four-state UI; verified cargo test --workspace and pnpm build
- **Next**: Run browser-level pagination, search, and filter checks for explore and services; then add API-level route tests for paginated responses

### [2026-09-28 15:31] main — Browser regression for pagination and full flow

- **Task**: Browser regression for pagination and full flow
- **Branch**: main
- **Summary**: Ran isolated browser regression for pagination, filters, route guards, full booking/payment/review flow, and paid cancellation refund. Fixed public list query deserialization that returned 400 and added API regression coverage. cargo test --workspace and pnpm build passed; main database fingerprint unchanged.
- **Next**: Run the regression suite after transaction, role, and creator profile changes; next add first-class customer/creator capabilities.

### [2026-09-28 15:34] main — Archive legacy code under legacy/

- **Task**: Archive legacy code under legacy/
- **Branch**: main
- **Summary**: Archived remaining legacy Flask runtime directories under legacy/flask_backend, verified the only active code roots are backend/ and web/, and passed cargo check plus pnpm build.
- **Next**: Keep legacy/ ignored as an archive; make the next structural pass by retiring root editor/cache directories only after confirming nothing active depends on them.

### [2026-09-28 16:34] main — Capability-based roles

- **Task**: Capability-based roles
- **Branch**: main
- **Summary**: Implemented capability-based roles across auth DTOs, service computation, frontend login routing, route guards, menu visibility, and role labels while preserving customer-to-creator onboarding. cargo test --workspace passed 36 tests; pnpm build passed 1846 modules. Commit pending.
- **Next**: Run browser-level three-role routing checks after roles changes; verify admin, creator, and customer login destinations plus forbidden route redirects.

### [2026-09-28 18:36] main — Address adversarial security and consistency findings

- **Task**: Address adversarial security and consistency findings
- **Branch**: main
- **Summary**: Fixed P0/P1 findings: mandatory strong JWT secret, authenticated and bounded AI chat with server-owned system prompt, atomic withdrawal review with conditional updates and Postgres row locking, unique email/phone conflicts with redacted internal errors, self-booking and duration validation, creator-only work publishing, inactive service hiding plus /services/mine, public DTO field removal, and active-user checks in auth middleware. Added regression tests across ai, common, services, and api. cargo test --workspace passed 35 tests; pnpm build passed 1846 modules; git diff --check passed.
- **Next**: Review the fixes with the adversarial supervisor, then run browser-level regression against the hardened auth, AI, service lifecycle, and withdrawal flows

### [2026-09-28 19:18] main — Adversarial re-review of security and consistency fixes

- **Task**: Independently verify commit 5b2a5570 and attempt bypasses
- **Branch**: main
- **Files**: web/scripts/audit_probe.py; web/scripts/test_runtime.py
- **Verification**: cargo test --workspace passed 45 tests; pnpm build passed 1846 modules; regression2 and regression3 E2E passed against isolated databases with unchanged main DB fingerprint. Audit confirmed mandatory JWT secret startup failure, forged-token rejection, authenticated/bounded AI chat, clean duplicate email/phone conflicts, self-booking rejection, creator-only works, inactive-service hiding, public DTO privacy, disabled-token rejection, and one-winner withdrawal review balance correctness.
- **Findings**: The duration check still accepts 120m59s because it compares truncated minutes; service creation accepts duration 30 and 600 even though appointment slots allow only 60-480; SQLite concurrent withdrawal review returns 500 (database is locked) instead of 409, although balance remains correct and the Postgres lock/conditional-update path is present.
- **Risks**: Postgres-specific row-lock behavior was reviewed statically but not executed because the isolated suite uses SQLite.
- **Next**: Make appointment duration comparison exact at second precision; validate service duration into 60-480 at creation/update; translate SQLite busy/lock errors during conditional review into Conflict or retry.

### [2026-09-28 19:28] main — Close remaining validation and concurrency gaps

- **Task**: Close remaining validation and concurrency gaps
- **Branch**: main
- **Summary**: Closed three adversarial-review findings: appointment duration now requires exact end-start equality so 12:00:59 cannot pass as 120 minutes; service creation rejects durations outside 60-480 minutes; SQLite lock, busy, and deadlock errors map to retryable 409 conflicts across the shared error boundary, with concurrent API review regression proving 200+409 and a single refund. cargo test --workspace passed 48 tests; pnpm build passed 1846 modules; git diff --check passed.
- **Next**: Submit for adversarial re-review after exact duration validation, service duration bounds, and SQLite deadlock-to-409 mapping

