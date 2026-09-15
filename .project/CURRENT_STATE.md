# Current State

**Branch**: main

## Active Work
- [x] Auth module: register/login/JWT

## Recent Changes
- backend/crates/api/src/middleware/auth.rs
- backend/crates/api/src/routes/auth.rs
- backend/crates/api/src/main.rs
- backend/crates/services/src/dto/auth.rs
- backend/crates/services/src/auth_service.rs
- backend/crates/db/src/entities/user.rs
- backend/crates/db/src/migrations/m20260915_000001_create_users.rs
- backend/crates/db/src/lib.rs
- backend/crates/core/src/entities/user.rs
- backend/crates/common/Cargo.toml

## Known Issues
- Rust MSVC linker missing — need VS Build Tools or GNU toolchain

## Next Step
- Creator profile + Service CRUD module
