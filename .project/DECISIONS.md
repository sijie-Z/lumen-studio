# Architecture Decisions

## ADR-001: 7-crate Rust workspace

**Reason**: Separation of concerns: core/db/services/api/ai/cache/common

**Impact**: All new code follows this crate layout

*2026-06-28 19:18*

## ADR-002: CRUD + audit log over Event Sourcing

**Reason**: Faster initial dev velocity, switch when stable

**Impact**: Domain events table exists but not primary write path

*2026-06-28 19:18*

## ADR-003: AI as visible user feature

**Reason**: Chat assistant, not just backend inference

**Impact**: Frontend needs AI chat component from day 1

*2026-06-28 19:18*

## ADR-004: GNU toolchain for Windows build

**Reason**: MSVC linker missing; GNU toolchain compiles cleanly

**Impact**: backend/rust-toolchain.toml pins stable-x86_64-pc-windows-gnu

*2026-09-15 22:16*

## ADR-005: SQLite for local dev, PostgreSQL for production

**Reason**: PostgreSQL 14 is running locally but superuser credentials are unknown; SeaORM supports both backends

**Impact**: DB URL switches via .env; migrations must stay compatible with both SQLite and PostgreSQL

*2026-09-15 22:20*

## ADR-006: New web/ SolidJS app replaces old frontend attempts

**Reason**: Old frontend and frontend_new are legacy reference code; clean web/ has SolidJS + UnoCSS + TanStack Query

**Impact**: All new UI work lives in web/; old frontend directories stay untouched as reference

*2026-09-15 23:30*

## ADR-007: Local upload storage before MinIO

**Reason**: MinIO is not running locally; uploads dir + ServeDir gives identical API surface

**Impact**: UploadResult.url is stable; swap storage layer later without changing UI

*2026-09-15 23:30*

## ADR-008: AI chat with local fallback + OpenAI-compatible upstream

**Reason**: Works immediately without a key; set OPENAI_API_KEY/BASE_URL/MODEL to switch to a real LLM

**Impact**: POST /api/v1/ai/chat returns mode local|llm; UI stays identical

*2026-09-15 23:35*

## ADR-009: Works persisted in SQLite/Postgres, no localStorage

**Reason**: Uploaded files were only browser-local; now stored in works table and listed publicly

**Impact**: POST/GET /api/v1/works; dashboard and home read the same source

*2026-09-15 23:47*

## ADR-010: Borrow shadcn token/variant patterns for SolidJS

**Reason**: React shadcn/Vercel templates can't be copied directly; adapt their design tokens and cva variant model into SolidJS + UnoCSS

**Impact**: ui/ component library is the single source of truth for styling

*2026-09-16 16:42*

## ADR-011: Appointment state machine + overlap detection

**Reason**: Enforce pending->confirmed->ongoing->completed with cancellations; reject overlapping slots per creator

**Impact**: Single source of truth in appointment_service; SQLite dev, Postgres prod

*2026-09-16 17:29*

## ADR-012: Seed real demo data + work categories

**Reason**: Replace smoke-test garbage with 8 creators, 8 services, 20 categorized works using 22 real Unsplash photos

**Impact**: Seed is idempotent; smoke tests must later use an isolated DB

*2026-09-16 21:32*

## ADR-013: Three role interfaces with seeded login accounts

**Reason**: admin/customer/creators each get a loginable account and a dedicated surface; login routes by role

**Impact**: Passwords: admin/admin123, customer/customer123, creators/creator123

*2026-09-17 07:16*

## ADR-014: Payment escrow + 10% commission settlement

**Reason**: Balance payment moves appointment to confirmed; creator completion triggers settlement with platform 10% commission

**Impact**: Escrow recorded in payments table; incomes verified in full-flow E2E

*2026-09-27 15:45*

## ADR-015: Withdrawal funds freeze on application

**Reason**: Prevents creators from spending the same balance while an admin review is pending; rejection restores the frozen amount atomically

**Impact**: withdrawals are pending/completed/rejected; approval records completion, rejection refunds to user balance

*2026-09-27 16:23*

## ADR-016: Best-effort in-app notifications

**Reason**: Notification delivery must not roll back an already successful appointment, payment, or withdrawal operation

**Impact**: Business routes call NotificationService after state changes; insert failures are logged without changing the primary API result

*2026-09-27 16:40*

## ADR-017: Grouped frontend layouts with route-level guards

**Reason**: Account, creator, and admin pages duplicated headers and performed request-failure-based redirects; public pages also carried their own navigation shell

**Impact**: web routes are grouped into public/customer/creator/admin layouts; AppShell owns navigation, notification, user menu, and mobile behavior; RequireAuth handles session hydration and RequireRole handles admin authorization before page render; creator onboarding remains available to authenticated customers until capabilities become a first-class role model

*2026-09-27 20:23*

## ADR-018: List pagination contract

**Reason**: PaginatedResponse is used for paginated public list APIs while legacy array helpers remain for older pages

**Impact**: New list screens use explicit page APIs and existing home/work/profile consumers stay compatible

*2026-09-28 00:58*

## ADR-019: Role capabilities are computed by auth service

**Reason**: Admin, customer, and creator access should be declared by the server instead of inferred from a creator list query in the frontend.

**Impact**: UserDto now includes roles for register/login/me; frontend guards and menus use role capabilities; raw role remains for JWT and admin middleware compatibility. Pure customers keep a Become Creator entry, and profile creation invalidates me so the capability upgrades without re-login.

*2026-09-28 16:33*

