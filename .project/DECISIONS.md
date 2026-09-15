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

