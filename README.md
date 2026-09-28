# Photography Service Platform

一个面向摄影与创意服务的预约交易平台原型。项目采用 Rust 构建后端领域与交易能力，SolidJS 构建前端体验，目标是逐步演进为一个成熟的多角色创意服务市场。

当前代码已经打通客户、创作者、管理员三类角色的核心业务闭环，但整体仍处于可运行、可测试的纵向 Demo 阶段，不应被当作已经完成生产加固的成品。

## 技术栈

### 后端

- Rust 2021
- Axum 0.8
- Tokio
- SeaORM 1.1
- SQLite（本地默认）
- PostgreSQL（生产目标，支持同一套 SeaORM 模型）
- Argon2 密码哈希
- JWT 鉴权
- tracing 日志
- Tower / tower-http

### 前端

- SolidJS
- TypeScript
- Vite
- UnoCSS
- `@solidjs/router`
- TanStack Solid Query
- Lucide Solid

### 测试与质量

- Rust workspace 单元测试和 API 集成测试
- Playwright Python 浏览器 E2E
- 隔离 SQLite 测试数据库
- 浏览器截图和控制台/API 错误捕获

## 架构概览

Rust workspace 位于 `backend/`，按职责拆分为以下 crate：

| Crate | 职责 |
| --- | --- |
| `common` | 统一错误、统一响应、分页契约、跨模块基础类型 |
| `app_core` | 领域枚举、值对象与核心业务规则 |
| `db` | SeaORM 实体、数据库连接和 migrations |
| `cache` | 缓存能力预留与基础设施适配 |
| `ai` | 可选的 AI 对话客户端、模型接口边界和本地回退逻辑 |
| `services` | 用例编排、事务边界、交易与领域服务 |
| `api` | Axum 路由、中间件、请求 DTO 和 HTTP 层 |

前端位于 `web/`，主要边界如下：

- `src/pages/`：公开发现页、客户中心、创作者工作台、管理后台
- `src/components/layout/`：按访问权限拆分的布局和路由守卫
- `src/components/ui/`：通用加载、空状态、错误状态和分页控件
- `src/lib/`：后端 API 客户端与 DTO
- `src/components/ai/`：可选的 AI 助手入口和对话界面

### 事务与事件边界

支付、退款、结算、提现审核等关键资金操作在服务层使用数据库事务，并通过条件更新和行锁降低并发重复操作风险。通知属于业务操作的附属效果，采用 best-effort 策略，通知失败不会伪装成主交易失败。

项目目标架构包含事件溯源和 CQRS，但当前实现尚未把 `domain_events` 作为所有写路径的唯一事实源。现阶段应先保证事务一致性、可测试性和清晰的领域边界，再逐步引入 Outbox、事件重放和独立读模型。

## 目录结构

```text
.
├── backend/
│   ├── Cargo.toml
│   ├── .env.example
│   └── crates/
│       ├── common/
│       ├── app_core/
│       ├── db/
│       ├── cache/
│       ├── ai/
│       ├── services/
│       └── api/
├── web/
│   ├── package.json
│   ├── vite.config.ts
│   ├── scripts/
│   └── src/
├── .project/
│   ├── CURRENT_STATE.md
│   ├── SESSION_LOG.md
│   ├── TASK_BOARD.md
│   └── DECISIONS.md
└── legacy/
    └── 旧版 Flask / Vue 代码归档（默认不提交）
```

## 已实现能力

### 三类角色

后端返回 capability 集合，而不是只依赖单一字符串角色：

- 普通客户：`["customer"]`
- 创作者：`["customer", "creator"]`
- 管理员：`["admin", "customer"]`
- 同时拥有创作者资料的管理员可拥有 `["admin", "customer", "creator"]`

前端根据 capability 控制登录跳转、导航入口和路由访问。

### 公开发现

- 首页与精选作品
- 作品探索、分类筛选、标题搜索和分页
- 服务列表、类型/地点/标题筛选和分页
- 创作者列表与创作者主页
- 作品详情与服务详情
- 服务详情到预约创建的流程入口

### 客户链路

- 注册、登录、刷新当前用户
- 账户余额和充值
- 创建预约
- 支付预约并进入平台托管
- 查看订单与支付流水
- 取消已支付预约并退款
- 完成预约后提交评价

### 创作者链路

- 创作者资料维护
- 作品上传与作品库
- 服务创建、启用和停用
- 查看收到的预约
- `confirmed -> ongoing -> completed` 履约状态推进
- 完成预约后触发结算与创作者入账
- 申请提现并查看提现记录

### 管理员链路

- 平台统计
- 用户列表
- 提现审核：通过或拒绝并返还冻结余额

### 平台能力

- 统一 API 响应和分页契约
- JWT 鉴权与管理员权限校验
- 用户状态检查，禁用账号的旧令牌立即失效
- 支付、退款、结算和提现审核的事务保护
- 充值、支付、提现审核相关通知
- 顶部通知铃铛、未读数和标记已读
- 客户、创作者、管理员 onboarding 引导
- 演示数据自动播种
- 本地演示图片资源

## 快速开始

### 前置要求

- Rust stable 工具链
- Node.js 20+
- pnpm
- Python 3.11+（仅在运行 Playwright E2E 时需要）

### 1. 启动后端

```powershell
cd backend
Copy-Item .env.example .env
```

编辑 `backend/.env`，至少确认以下配置：

```dotenv
DATABASE_URL=sqlite://photography.db?mode=rwc
JWT_SECRET=请替换为至少32字符的随机密钥
BIND_ADDR=127.0.0.1:8080
RUST_LOG=api=debug,tower_http=debug
```

`JWT_SECRET` 缺失、为空或少于 32 字符时，API 会拒绝启动。不要在生产环境使用仓库中的示例值。

然后运行：

```powershell
cargo run -p api
```

后端默认监听 `http://127.0.0.1:8080`。

健康检查：

```powershell
Invoke-WebRequest http://127.0.0.1:8080/health
```

### 2. 启动前端

打开新的 PowerShell 窗口：

```powershell
cd web
pnpm install
pnpm dev
```

前端默认监听 `http://127.0.0.1:5173`。Vite 会把 `/api` 和 `/uploads` 代理到后端 `8080` 端口。

### 3. 登录演示账号

首次启动后端时会自动写入演示数据：

| 角色 | 用户名 | 密码 |
| --- | --- | --- |
| 管理员 | `admin` | `admin123` |
| 普通客户 | `customer` | `customer123` |
| 创作者示例 | `chenyu` | `creator123` |

其他演示创作者也使用密码 `creator123`，包括：

- `linye`
- `suhe`
- `zhoumo`
- `guchuan`
- `xuyan`
- `shenlu`
- `hanche`

这些账号仅用于本地开发，不应出现在生产环境。

## 核心业务闭环

```text
客户浏览作品/服务
  -> 分类、筛选与搜索发现
  -> 选择创作者与服务
  -> 创建预约
  -> 客户充值并支付
  -> 金额进入平台托管
  -> 创作者确认并履约
  -> 预约完成
  -> 平台按 10% 佣金结算
  -> 创作者申请提现
  -> 管理员审核
  -> 客户评价
```

关键资金路径：

```text
充值 -> 客户余额
支付 -> 客户余额扣除，平台托管
完成 -> 平台抽佣，创作者收入增加
提现申请 -> 创作者余额冻结
提现拒绝 -> 冻结金额返还
提现通过 -> 记录完成状态
已支付预约取消 -> 客户余额退款
```

## 测试

### Rust workspace

```powershell
cd backend
cargo test --workspace
```

该命令覆盖服务层、API 集成、鉴权、支付、退款、结算、提现、通知、评价、分页和并发保护相关测试。

### 前端构建

```powershell
cd web
pnpm build
```

### 浏览器 E2E

先安装 Playwright Python 依赖：

```powershell
python -m pip install playwright
python -m playwright install chromium
```

部分回归脚本使用系统 Chrome 的 `chrome` channel。Windows 环境需要已安装 Chrome。

常用脚本：

```powershell
python web/scripts/regression2_e2e.py
python web/scripts/regression3_e2e.py
python web/scripts/onboarding_notification_e2e.py
python web/scripts/withdrawal_e2e.py
python web/scripts/audit_probe.py
```

这些脚本通过 `web/scripts/test_runtime.py` 自动启动隔离的测试数据库、Rust API 和 Vite，并在结束后清理进程。运行前应确保 `8080` 和 `5173` 端口空闲。

其他可用脚本：

- `e2e_flow.py`：主交易流程
- `regression_e2e.py`：早期三角色与核心流程回归
- `smoke_ui.py`：基础 UI 冒烟测试

## 配置说明

后端读取 `backend/.env`：

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `DATABASE_URL` | `sqlite://photography.db?mode=rwc` | SeaORM 数据库连接地址 |
| `JWT_SECRET` | 无 | 必填，至少 32 字符 |
| `BIND_ADDR` | `127.0.0.1:8080` | API 监听地址 |
| `UPLOAD_DIR` | `uploads` | 上传图片目录 |
| `RUST_LOG` | `api=debug,tower_http=debug` | tracing 日志过滤 |

可选的 AI 对话接口可通过环境配置接入 OpenAI 兼容服务；未配置外部服务时使用本地回退回复。AI 助手不是平台核心能力，也不参与预约、支付、结算和履约主流程。

## 设计取舍

- 先做纵向可运行闭环，再扩展平台规模。
- 关键写路径优先使用事务保证一致性，不依赖前端按钮状态防止重复操作。
- 角色使用 capability 集合，允许客户后续升级为创作者。
- API 统一使用响应包裹和分页契约，前端通过类型化客户端调用。
- AI 助手定位为可选附加能力；平台核心仍是摄影 / 创意服务的发现、预约、交易、履约和评价闭环。
- 旧 Flask / Vue 代码仅作为业务参考归档在 `legacy/`，当前有效入口是 `backend/` 和 `web/`。

## 已知限制

- 当前更接近可运行、可测试的纵向 Demo，不是已经完成生产加固的产品。
- PostgreSQL 的行锁和并发路径已经实现并经过静态审查，但仍需要在真实 PostgreSQL 测试环境中执行集成测试。
- 事件溯源和 CQRS 是目标架构，目前尚未覆盖全部写路径。
- 可选的 AI 助手已接入鉴权、输入限制和可配置模型接口，但不是平台核心；业务工具调用、档期查询、创建预约和向量检索仍未实现。
- 提现“通过”当前记录为已完成打款状态，尚未对接真实支付渠道。
- 浏览器回归依赖本机 Python、Playwright 和部分脚本所需的系统 Chrome。
- 当前 CORS、限流和部署配置仍需要按生产环境进行收紧。

## 开发约定

- 代码修改必须经过测试或明确说明未验证原因。
- 重要架构决策写入 `.project/DECISIONS.md`。
- 每次开发会话追加 `.project/SESSION_LOG.md`。
- 任务状态维护在 `.project/TASK_BOARD.md`。
- 新会话开始前阅读 `.project/CURRENT_STATE.md`。
- 不提交数据库、上传文件、构建产物和 `legacy/` 归档。

## License

当前仓库尚未声明开源许可证。在添加明确的 `LICENSE` 文件前，默认保留所有权利。
