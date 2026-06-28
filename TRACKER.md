## TRACKER — 项目进度锚

> 每次结束工作前必须更新此文件。下次打开项目先读这个。

---

### 当前状态 (2026-06-28)

**所在分支**: `main` (初始化阶段)

**刚完成**：
- 项目从零初始化：git init, TRACKER.md, .gitignore
- 从老代码中提取了核心业务规则（状态机、枚举、校验逻辑）
- Rust 工作空间骨架搭建中

**下一步**：
- 完成 core crate：值对象 (Money, Email, Phone, TimeSlot) + 枚举 + 实体定义
- 完成 db crate：SeaORM 模型 + migration
- auth 模块 (注册/登录/JWT)

**关键架构决策**：
1. 暂不使用事件溯源，用常规 CRUD + 审计日志 — 等核心业务稳定后切换
2. Rust 工作空间 7 crate 结构（core / db / services / api / ai / cache / common）
3. AI 功能作为可见的用户端功能（AI 助手聊天），不是纯后台推理
4. 优先打通 auth → creator → appointment → payment 价值链路
5. 每个大模块做完后合并到 main，打 tag

**分支策略**：
- `main` — 稳定主线
- `feat/<module>` — 功能分支（如 feat/auth, feat/appointment）
- 每个功能分支完成后 squash merge 到 main

---

### 历史记录

#### 2026-06-28 — 初始化

- git init
- 从 backend/app/common/enums.py 提取状态枚举
- 从 backend/app/services/* 提取业务规则
- 从 backend/app/models/index.py 提取数据模型设计
- 创建 TRACKER.md
- Rust workspace 搭建中
