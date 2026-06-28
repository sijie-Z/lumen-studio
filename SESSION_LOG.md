## SESSION_LOG — 全会话开发记录

> 每个 CLI 窗口结束工作前必须追记一条。打开项目先读此文件末尾 30 行。
> 格式：`### [日期 时间] 分支 — 一句话概括`

---

### [2026-06-28 17:30] main — 项目初始化，Rust workspace + core domain layer

- **分支**: main (初始提交)
- **Commit**: `fc44728` (feat: Rust workspace + core domain layer)
- **Tag**: `v0.1.0-core`
- **新增文件 (25)**: 7 crate Cargo.toml + common(error/response) + core(enums/value_objects/entities)
- **关键决策**:
  1. 7-crate workspace (common/core/db/services/api/cache/ai)
  2. 暂不启用事件溯源，用 CRUD + 审计日志
  3. AI 作为用户可见的聊天助手，非纯后台推理
  4. 优先打通 auth → creator → appointment → payment 价值链路
  5. 每个大模块 squash merge 到 main + annotated tag
- **阻塞项**: Rust MSVC linker 未安装，需 VS Build Tools 或 GNU toolchain
- **下一步**: feat/auth — 数据库迁移 + 注册/登录/JWT
