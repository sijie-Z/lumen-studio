## TRACKER — 项目进度锚

> 每次结束工作前必须更新此文件。下次打开项目先读这个。

---

### 当前状态 (2026-06-28)

**所在分支**: main
**最后提交**: c44728 — feat: Rust workspace + core domain layer

**刚完成**：
- git init + first commit (25 files, 758 loc)
- Rust 工作空间 7-crate 结构搭建完成，399 个依赖全部解析通过
- common crate：AppError (10 种错误类型) + ApiResponse + PaginatedResponse
- core crate：10 种状态枚举（含 AppointmentStatus 状态机转换验证）
- core crate：4 个值对象（Money/Email/Phone/TimeSlot）— 编译期类型安全
- core crate：13 个领域实体 struct（User/Appointment/Payment/Service/Portfolio/...）
- db/ai/cache/services crate：stub 占位，等待下个模块填充

**下一步**：
- **解决编译环境**：安装 VS Build Tools 或换 GNU toolchain（当前唯一阻塞项）
- db crate：SeaORM 活跃模型 + migration（优先建 users/appointments/services 表）
- services crate：auth 模块（注册/登录/JWT）
- feat/auth 分支开始实现

**关键架构决策 (5条)**：
1. 暂不使用事件溯源，用常规 CRUD + 审计日志 — 等核心业务稳定后切换
2. Rust 工作空间 7 crate 结构（core / db / services / api / ai / cache / common）
3. AI 功能作为可见的用户端功能（AI 助手聊天），不是纯后台推理
4. 优先打通 auth → creator → appointment → payment 价值链路
5. 每个大模块做完后合并到 main，打 tag

**分支策略**：
- main — 稳定主线
- eat/<module> — 功能分支
- 每个 feature 分支完成后 squash merge 到 main，打 annotated tag

---

### 历史记录

#### 2026-06-28 — 初始化

- git init (移除 backend/.git 嵌套仓库)
- 从 backend/app/common/enums.py 提取状态枚举
- 从 backend/app/services/* 提取业务规则
- 从 backend/app/models/index.py 提取数据模型设计
- 创建 TRACKER.md + .gitignore
- 搭建 Rust workspace (7 crates, 399 deps)
- 创建 core domain layer (enums + value objects + entities)
- Commit: c44728 (25 files, 758 loc)
