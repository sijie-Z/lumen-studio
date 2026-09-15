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

