# Photography AI Platform — Architecture Bible

> 从一个摄影预约系统出发，构建一个 **AI 原生的创意服务平台**。
> 摄影是切入点，平台支持一切创意服务：摄影、摄像、化妆、造型、设计。
> Rust 做骨骼，SolidJS 做皮肤，AI 做神经系统。
> 目标：教科书级别的全栈工程实践。

---

## 一、项目定位

### 1.1 这是什么

一个创意服务交易平台，类似 500px + 美团 + Fiverr 的融合体，但 AI 原生。

**供给侧（创作者）**：摄影师、摄像师、化妆师、造型师、设计师——发布服务、展示作品、管理预约、收款提现。

**需求侧（客户）**：浏览作品、智能匹配、预约服务、在线支付、评价反馈。

**AI 层**：贯穿全流程——风格匹配、智能搜索、质量评估、自动修图、Agent 对话、需求预测、动态定价。

### 1.2 和现有项目的关系

现有项目（Flask + Vue 3）的业务逻辑是成熟的：
- 35+ 数据模型，含完整审计追踪
- 70+ API 端点，覆盖预约/支付/交付/评价全流程
- AI 图片处理管线（GFPGAN、RealESRGAN）
- 财务系统（充值、支付、提现、佣金）
- 角色路由（客户、创作者、管理员、配送员）
- 深色/浅色双主题 UI，毛玻璃设计语言

**保留**：业务模型、状态机、审计追踪、财务逻辑、AI 管线设计。
**重写**：技术栈（Flask→Rust，Vue→SolidJS）、架构模式（→事件溯源+CQRS）、AI 能力（→向量检索+Agent）。

### 1.3 为什么是 Rust + SolidJS

这两个框架有同一个信仰：**零开销抽象**。

Rust 的所有权系统在编译期消除内存管理开销——没有 GC，没有引用计数，没有泄漏。运行时只做必要的事。

SolidJS 的 Signals 在编译期消除 Virtual DOM diff——没有 reconciliation，没有不必要的重渲染。DOM 更新只发生在依赖变化的精确位置。

前后端用同一种思维方式构建：**只做必要的事，零多余开销**。

### 1.4 为什么 Python 做 AI 侧车

Python 是 AI 的母语。PyTorch、Transformers、CLIP、LangChain——整个生态都在 Python 里。

但我们不把 Python 放在核心路径上。Rust 处理并发、事务、缓存、WebSocket；Python 只做 GPU 推理。两者通过 gRPC 通信。

Rust 不碰 GPU，Python 不碰事务。各司其职。

### 1.5 为什么事件溯源

传统 CRUD 只存最终状态：
```sql
UPDATE appointments SET status = 'confirmed' WHERE id = 1;
-- 只知道"现在是什么"，不知道"经历了什么"
```

事件溯源记录每一步发生了什么：
```
Event 1: AppointmentCreated   {user:1, photographer:5, price:3999}
Event 2: PaymentReceived      {amount:3999, method:"alipay"}
Event 3: AppointmentConfirmed {by:photographer_5}
Event 4: DeliveryCompleted    {files:48}
Event 5: ReviewSubmitted      {rating:5}
```

→ 完整业务时间线，任意时间点可重建，审计/纠纷/分析天然支持。

### 1.6 为什么 AI 是核心能力

不是"加了个 /api/ai 接口"。AI 渗透到系统的每一个角落：

- 每张作品上传 → 自动提取风格向量 → 存入向量库
- 客户搜索"日系小清新" → CLIP 编码 → 向量匹配最契合的创作者
- 预约创建 → 智能调度综合天气/交通/负载给出最优时段
- 客户对话 → Agent 理解意图 → 调用工具 → 自然语言回复
- 管理后台 → 需求预测/动态定价/用户分群/归因分析

---

## 二、技术栈

### 2.1 后端：Rust

| 层 | 选型 | 说明 |
|----|------|------|
| Web 框架 | **Axum 0.8** | Tokio 团队出品，Tower 中间件生态 |
| 运行时 | **Tokio** | Rust 异步运行时标准 |
| ORM | **SeaORM 1.1** | 异步，动态查询，migration |
| 数据库 | **PostgreSQL 16** | JSONB 可索引，全文搜索，SKIP LOCKED |
| 缓存/消息 | **Redis 7** | 缓存、分布式锁、Pub/Sub、Streams 任务队列 |
| 向量库 | **Qdrant** | 风格向量检索、语义搜索、推荐 |
| 全文搜索 | **Meilisearch** | 中文分词，typo-tolerant |
| 分析 | **ClickHouse** | 列式存储，OLAP，亿级秒查 |
| 对象存储 | **MinIO** | S3 兼容，图片/视频 |
| AI 通信 | **Tonic (gRPC)** | Rust ↔ Python 高性能通信 |
| 序列化 | **Serde** | JSON 零成本序列化 |
| JWT | **jsonwebtoken** | 签发/验证 |
| 密码 | **argon2** | 比 bcrypt 更安全 |
| 错误处理 | **thiserror + anyhow** | 业务错误 + 系统错误 |
| 配置 | **dotenvy + serde** | 类型安全配置 |
| 日志 | **tracing** | 结构化日志 + OpenTelemetry |
| 限流 | **governor** | 令牌桶 |
| CORS | **tower-http** | 中间件 |
| HTTP 客户端 | **reqwest** | 调外部 API |
| 金额 | **rust_decimal** | 精确十进制运算 |

### 2.2 前端：SolidJS

| 层 | 选型 | 说明 |
|----|------|------|
| 框架 | **SolidJS** | Signal-based 细粒度响应式 |
| 语言 | **TypeScript** | 严格类型 |
| 构建 | **Vite** | 极速 HMR |
| 路由 | **@solidjs/router** | 类型安全路由 |
| 数据获取 | **@tanstack/solid-query** | 异步状态 + 缓存 + 重试 |
| 状态 | **Solid Signals** | 原生信号，无额外库 |
| CSS | **UnoCSS** | 原子化，按需生成 |
| UI 组件 | **Kobalte** | 无障碍组件库 |
| 表格 | **@tanstack/solid-table** | 无头表格 |
| 表单 | **@modular-forms/solid** | 类型安全表单 |
| 图表 | **ECharts** | 数据看板 |
| Markdown | **solid-markdown** | AI 对话渲染 |
| API Client | **openapi-typescript-codegen** | 从后端 OpenAPI 自动生成 |
| 工具 | **@solid-primitives** | SolidJS 原语工具库 |

### 2.3 AI 服务：Python

| 层 | 选型 | 说明 |
|----|------|------|
| RPC | **gRPC + Tonic** | 高性能二进制通信 |
| ML | **PyTorch 2.x** | GPU 推理 |
| 视觉 | **CLIP (ViT-L/14)** | 图文多模态嵌入 |
| 质量 | **自训练模型** | 清晰度/曝光/构图 |
| 修图 | **GFPGAN + Real-ESRGAN** | 人脸修复 + 超分 |
| 抠图 | **SAM** | 通用分割 |
| LLM | **Qwen 2.5 / Claude API** | Agent、内容生成 |
| Agent | **Function Calling** | 工具调用 |
| 嵌入 | **sentence-transformers** | 文本向量化 |

### 2.4 基础设施

| 组件 | 选型 | 说明 |
|------|------|------|
| 网关 | **Nginx** | SSL、限流、安全头、反向代理 |
| 容器 | **Docker + Compose** | 多阶段构建，一键启动 |
| CI/CD | **GitHub Actions** | 自动测试/构建/部署 |
| 指标 | **Prometheus + Grafana** | 请求量/延迟/错误率 |
| 日志 | **Loki + Grafana** | 结构化日志聚合 |
| 追踪 | **OpenTelemetry** | 全链路追踪 |
| CDN | **Cloudflare** | 图片加速、DDoS 防护 |

---

## 三、架构模式

### 3.1 事件溯源 + CQRS

**写入侧**：所有变更以事件形式 append 到 `domain_events` 表（不可变、append-only）。

**读取侧**：物化视图可以有独立的查询模型，针对不同查询场景优化。

```
Command (写入)                    Query (读取)
    │                                 │
    ▼                                 ▼
┌─────────┐                    ┌──────────┐
│ Service  │                    │ Materialized│
│ 业务逻辑 │                    │  View     │
└────┬─────┘                    └─────┬────┘
     │                                │
     ▼                                │
┌─────────┐                           │
│  Event   │──── 重放事件 ───────────→│
│  Store   │     构建物化视图          │
└─────────┘                           │
     │                                │
     ▼                                │
┌─────────┐                           │
│  Redis   │──── Pub/Sub 广播 ────→ WebSocket 推送
│  Pub/Sub │                           │
└─────────┘                           │
```

**实现细节**：
- `domain_events` 表：`id, aggregate_type, aggregate_id, event_type, event_data (JSONB), version, created_at, created_by`
- 乐观锁：`version` 字段防止并发写入冲突
- 聚合根通过重放事件重建状态
- 物化视图可以独立更新，最终一致性

### 3.2 值对象（编译期业务规则）

用 Rust 类型系统在编译期强制业务规则：

```rust
// 不是"一个浮点数代表金额"，是 Money 类型
// 编译期保证：不能为负、精度不超过2位小数、不能和字符串相加
pub struct Money(Decimal);

// 不是"两个 DateTime 代表时段"，是 TimeSlot 类型
// 编译期保证：结束 > 开始、时长 1-8 小时、不能是过去
pub struct TimeSlot { start: DateTime<Utc>, end: DateTime<Utc> }

// 不是"一个 String 代表邮箱"，是 Email 类型
// 编译期保证：格式合法
pub struct Email(String);

// 不是"一个 String 代表手机号"，是 Phone 类型
// 编译期保证：格式合法
pub struct Phone(String);
```

### 3.3 分布式锁（并发安全）

预约、支付等关键操作用 Redis 分布式锁：

```
创建预约流程:
  1. 获取锁 lock:slot:{photographer}:{date}:{time}
     → Redis SET NX + EXPIRE，Lua 脚本保证原子性
  2. 检查时段是否已被预约
  3. 创建预约事件（append to event store）
  4. 发布事件到 Redis Pub/Sub
  5. 释放锁（Drop 自动释放，Lua 脚本保证只删自己的锁）
  6. WebSocket 推送通知给创作者

锁的安全性：
  - 过期时间防止死锁
  - UUID 标识防止误删
  - Lua 脚本保证 check-and-delete 原子性
```

### 3.4 多级缓存

```
请求 → Nginx 缓存 (静态资源/CDN)
     → Redis 缓存 (热数据，TTL 5min)
     → PostgreSQL (持久化)

缓存策略：
  - 读多写少的数据（创作者资料、服务列表）→ Redis 缓存
  - 写入时主动失效（Cache-Aside 模式）
  - 事件驱动失效：事件发布时清除相关缓存
  - 缓存预热：系统启动时加载热门数据
```

### 3.5 API 版本化

```
/api/v1/users      ← 当前版本
/api/v2/users      ← 新版本（breaking change 时）

版本切换策略：
  - 新版本发布后，旧版本保留 6 个月
  - 通过 Nginx 路由到不同后端实例
  - 前端通过配置切换 API 版本
```

### 3.6 统一响应格式

```json
// 成功
{ "code": 200, "message": "success", "data": { ... } }

// 分页
{
  "code": 200,
  "message": "success",
  "data": {
    "items": [ ... ],
    "total": 100,
    "page": 1,
    "page_size": 20,
    "has_next": true
  }
}

// 错误
{ "code": 409, "message": "该时段已被预约" }
```

---

## 四、六大引擎

### 4.1 AI Agent 引擎

不是聊天机器人，是**有手有脚的智能体**。

```python
tools = [
    # 搜索与发现
    search_photographers,       # 根据风格/预算/地点搜索创作者
    search_services,            # 搜索服务
    get_style_recommendations,  # 风格推荐

    # 预约管理
    check_availability,         # 查档期
    create_appointment,         # 创建预约
    reschedule_appointment,     # 改期
    cancel_appointment,         # 取消预约

    # 信息查询
    get_creator_profile,        # 查看创作者详情
    get_service_detail,         # 查看服务详情
    get_price_estimate,         # 价格预估
    check_weather,              # 查天气

    # 内容生成
    generate_mood_board,        # 生成灵感板
    generate_description,       # 生成作品描述
    generate_social_copy,       # 生成社交文案

    # 业务分析 (创作者/管理员)
    get_business_analytics,     # 经营数据
    get_demand_forecast,        # 需求预测
    get_pricing_suggestion,     # 定价建议
]
```

**Agent 能力**：
- 理解模糊需求："我想拍那种阳光洒在脸上的感觉" → 解析为风格向量 → 匹配
- 多轮对话：记住上下文，逐步细化需求
- 自主决策：根据用户意图选择调用哪些工具
- 流式输出：实时返回结果，不等全部完成
- 记忆系统：记住用户偏好、历史对话、风格偏好

### 4.2 智能搜索引擎

三层融合：

```
用户查询: "浪漫的、有海的、暖色调的婚纱照"
                │
                ▼
┌───────────────────────────────────┐
│ 第一层: CLIP 语义编码              │
│ "浪漫的海景暖色婚纱" → 512维向量   │
│ 在 Qdrant 中检索语义最相似的作品   │
└───────────────┬───────────────────┘
                │
┌───────────────▼───────────────────┐
│ 第二层: Meilisearch 全文检索       │
│ 中文分词 + typo-tolerant           │
│ 关键词精确匹配                     │
└───────────────┬───────────────────┘
                │
┌───────────────▼───────────────────┐
│ 第三层: 个性化重排                 │
│ 结合用户历史偏好、评分、距离、价格 │
│ Learning-to-Rank 模型              │
└───────────────┬───────────────────┘
                │
                ▼
          最终排序结果
```

### 4.3 智能调度引擎

不只是"选个时间"，是**全局最优编排**：

```python
factors = {
    "creator_workload":       # 当日已排单数，避免过劳
    "location_clustering":    # 地理聚类，同区域排一起
    "weather_forecast":       # 外景调天气 API，雨天建议室内
    "golden_hour":            # 日出日落，外景黄金时段优先
    "traffic_prediction":     # 交通预估，留出通勤时间
    "client_preferences":     # 客户偏好时段
    "seasonal_demand":        # 季节性需求，旺季动态定价
    "creator_rating":         # 高评分创作者优先推荐
}
```

### 4.4 风格 DNA 系统

每张作品上传时，自动分析并存储风格特征：

```
图片上传
    │
    ▼
┌─────────────────────────────────┐
│ CLIP ViT-L/14 特征提取           │
│ 提取 768 维风格向量               │
└───────────────┬─────────────────┘
                │
    ┌───────────┼───────────┐
    ▼           ▼           ▼
┌───────┐ ┌─────────┐ ┌─────────┐
│色调   │ │构图     │ │情绪     │
│暖/冷  │ │居中/三分│ │浪漫/纪实│
│饱和度 │ │留白/紧凑│ │活泼/沉稳│
└───────┘ └─────────┘ └─────────┘
    │           │           │
    └───────────┼───────────┘
                ▼
        存入 Qdrant (向量库)
        关联创作者 ID
        支持实时检索
```

### 4.5 实时通信层

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Rust 后端   │     │  Redis       │     │  WebSocket   │
│              │     │  Pub/Sub     │     │  连接池      │
│ 事件发布 ────┼────→│ 频道广播 ────┼────→│ 精准推送     │
└──────────────┘     └──────────────┘     └──────────────┘

WebSocket 事件类型:
  - appointment:created       → 创作者收到新预约
  - appointment:confirmed     → 客户收到预约确认
  - appointment:cancelled     → 对方收到取消通知
  - payment:received          → 创作者收到收款通知
  - delivery:started          → 客户收到交付开始
  - delivery:completed        → 客户收到交付完成
  - review:submitted          → 创作者收到新评价
  - notification:new          → 通用通知
```

### 4.6 数据分析平台

| 分析维度 | 具体能力 |
|----------|---------|
| 需求预测 | 历史数据 + 季节 + 节假日 → 预测未来 30 天各类型需求量 |
| 动态定价 | 旺季/周末/热门创作者自动溢价，淡季折扣，最大化收入 |
| 用户分群 | RFM 模型：高价值/流失风险/新客户，差异化运营 |
| 创作者画像 | 接单率/准时率/好评率/退款率 → 信用分 |
| 归因分析 | 客户来源渠道 ROI |
| 热力图 | 城市/区域/时段需求热力图 |
| 流失预警 | 用户行为模式变化 → 预测流失 → 触发挽回 |

技术：ClickHouse 列式存储，亿级数据秒级查询。

---

## 五、业务领域

### 5.1 用户角色

| 角色 | 能力 |
|------|------|
| **客户** | 浏览、搜索、预约、支付、评价、收藏、AI 助手 |
| **创作者** | 发布服务、管理作品集、处理预约、收款提现、数据分析 |
| **管理员** | 用户管理、认证审核、内容审核、财务管理、数据统计 |
| **配送员** | 实物交付任务管理 |

### 5.2 核心业务流

```
客户浏览 → AI 匹配 → 查看作品集 → 选择服务 → 创建预约
    → 支付 → 创作者确认 → 拍摄执行 → 交付作品
    → 客户验收 → 评价 → 创作者收款
```

### 5.3 状态机

**预约状态**：
```
pending → confirmed → ongoing → completed
   ↓         ↓          ↓
cancelled  cancelled  cancelled → refunded
```

**支付状态**：
```
pending → success → (自动确认预约)
   ↓
failed / expired(30min)
```

**认证状态**：
```
unverified → pending → verified
                ↓
             rejected
```

**提现状态**：
```
pending → approved → completed
   ↓
rejected
```

### 5.4 财务系统

- 客户充值（支付宝/微信）→ 账户余额
- 预约支付 → 从余额扣除 → 平台托管
- 服务完成 → 平台抽佣 → 创作者可提现
- 提现申请 → 管理员审核 → 打款
- 退款 → 退回客户余额

**双重记账**：每一笔资金流动都有借方和贷方，总额永远平衡。

### 5.5 AI 能力

| 能力 | 说明 |
|------|------|
| 风格 DNA | CLIP 提取作品风格向量，建立创作者风格画像 |
| 智能匹配 | 客户描述需求 → 向量匹配最契合的创作者 |
| 质量评估 | 自动评分：清晰度/曝光/构图 |
| AI 修图 | GFPGAN 人脸修复 + Real-ESRGAN 超分 |
| 智能抠图 | SAM 通用分割 |
| Agent 对话 | LLM + Function Calling，自然语言预约 |
| 内容生成 | 自动生成作品描述、社交文案 |
| 需求预测 | 历史数据 + 季节 + 节假日预测需求 |
| 动态定价 | 旺季溢价、淡季折扣 |

---

## 六、数据库设计

### 6.1 核心表

```sql
-- 用户表
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash VARCHAR(256) NOT NULL,
    nickname VARCHAR(50) DEFAULT '匿名用户',
    avatar_url VARCHAR(255),
    gender VARCHAR(10) DEFAULT 'unknown',
    date_of_birth DATE,
    bio TEXT,
    email VARCHAR(120) UNIQUE,
    phone VARCHAR(20) UNIQUE,
    balance DECIMAL(10,2) DEFAULT 0.00,
    status VARCHAR(20) DEFAULT 'active',
    role VARCHAR(20) DEFAULT 'user',
    verification_status VARCHAR(20) DEFAULT 'unverified',
    verification_message TEXT,
    verification_time TIMESTAMPTZ,
    real_name VARCHAR(50),
    id_card VARCHAR(20) UNIQUE,
    last_login_time TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 创作者资料表
CREATE TABLE creator_profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER UNIQUE REFERENCES users(id),
    introduction TEXT,
    bio TEXT,
    rating DECIMAL(2,1) DEFAULT 5.0,
    certification_level VARCHAR(20) DEFAULT 'standard',
    service_areas JSONB,
    available_slots JSONB,
    style_vector_id VARCHAR(64),
    portfolio_url VARCHAR(255),
    total_services INTEGER DEFAULT 0,
    total_appointments INTEGER DEFAULT 0,
    total_income DECIMAL(10,2) DEFAULT 0.00,
    avg_rating DECIMAL(3,2) DEFAULT 0.00,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 服务类型表
CREATE TABLE service_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 服务表
CREATE TABLE services (
    id SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES creator_profiles(id),
    type_id INTEGER REFERENCES service_types(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10,2) NOT NULL,
    duration INTEGER,
    cover_image_url VARCHAR(255),
    location VARCHAR(255),
    tags VARCHAR(255),
    options JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    is_featured BOOLEAN DEFAULT FALSE,
    appointments_count INTEGER DEFAULT 0,
    monthly_appointments INTEGER DEFAULT 0,
    quarterly_appointments INTEGER DEFAULT 0,
    yearly_appointments INTEGER DEFAULT 0,
    style_vector_id VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 预约表
CREATE TABLE appointments (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    creator_id INTEGER REFERENCES creator_profiles(id),
    service_id INTEGER REFERENCES services(id),
    appointment_date DATE NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    location VARCHAR(255),
    status VARCHAR(20) DEFAULT 'pending',
    total_price DECIMAL(10,2),
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 支付表
CREATE TABLE payments (
    id SERIAL PRIMARY KEY,
    appointment_id INTEGER REFERENCES appointments(id),
    user_id INTEGER REFERENCES users(id),
    amount DECIMAL(10,2) NOT NULL,
    method VARCHAR(20),
    status VARCHAR(20) DEFAULT 'pending',
    tx_id VARCHAR(64),
    expire_time TIMESTAMPTZ,
    payment_channel VARCHAR(20),
    refund_amount DECIMAL(10,2),
    refund_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 作品集表
CREATE TABLE portfolios (
    id SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES creator_profiles(id),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    cover_image_url VARCHAR(255),
    category VARCHAR(50),
    is_public BOOLEAN DEFAULT TRUE,
    likes_count INTEGER DEFAULT 0,
    style_vector_id VARCHAR(64),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 作品表
CREATE TABLE works (
    id SERIAL PRIMARY KEY,
    portfolio_id INTEGER REFERENCES portfolios(id),
    image_url VARCHAR(255) NOT NULL,
    description TEXT,
    width INTEGER,
    height INTEGER,
    aspect_ratio VARCHAR(10),
    style_vector_id VARCHAR(64),
    quality_score DECIMAL(3,2),
    style_tags JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 评价表
CREATE TABLE reviews (
    id SERIAL PRIMARY KEY,
    appointment_id INTEGER REFERENCES appointments(id),
    user_id INTEGER REFERENCES users(id),
    creator_id INTEGER REFERENCES creator_profiles(id),
    rating DECIMAL(2,1) NOT NULL,
    content TEXT,
    images JSONB,
    is_anonymous BOOLEAN DEFAULT FALSE,
    photographer_reply TEXT,
    replied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 事件存储表（核心）
CREATE TABLE domain_events (
    id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(50) NOT NULL,
    aggregate_id INTEGER NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    version INTEGER NOT NULL,
    created_by INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB,
    UNIQUE(aggregate_type, aggregate_id, version)
);

-- 通知表
CREATE TABLE notifications (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    type VARCHAR(20),
    title VARCHAR(255),
    content TEXT,
    priority VARCHAR(10) DEFAULT 'normal',
    action_url VARCHAR(255),
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 充值记录表
CREATE TABLE prepaids (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    amount DECIMAL(10,2) NOT NULL,
    payment_method VARCHAR(20),
    platform VARCHAR(20),
    tx_id VARCHAR(64),
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 提现记录表
CREATE TABLE withdrawals (
    id SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES creator_profiles(id),
    amount DECIMAL(10,2) NOT NULL,
    fee DECIMAL(10,2) DEFAULT 0.00,
    actual_amount DECIMAL(10,2),
    status VARCHAR(20) DEFAULT 'pending',
    account_info JSONB,
    reviewed_by INTEGER,
    reviewed_at TIMESTAMPTZ,
    review_note TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 收藏表
CREATE TABLE favorites (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    target_type VARCHAR(20),
    target_id INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, target_type, target_id)
);

-- 点赞表
CREATE TABLE likes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    portfolio_id INTEGER REFERENCES portfolios(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, portfolio_id)
);

-- 轮播图表
CREATE TABLE rotations (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255),
    image_url VARCHAR(255),
    link_url VARCHAR(255),
    sort_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 公告表
CREATE TABLE announcements (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255),
    content TEXT,
    priority VARCHAR(10) DEFAULT 'normal',
    expires_at TIMESTAMPTZ,
    published_by INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 富文本表
CREATE TABLE rich_texts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255),
    content TEXT,
    content_type VARCHAR(20) DEFAULT 'html',
    entity_type VARCHAR(50),
    entity_id INTEGER,
    status VARCHAR(20) DEFAULT 'draft',
    view_count INTEGER DEFAULT 0,
    published_by INTEGER,
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 交付表
CREATE TABLE deliveries (
    id SERIAL PRIMARY KEY,
    appointment_id INTEGER REFERENCES appointments(id),
    creator_id INTEGER REFERENCES creator_profiles(id),
    delivery_type VARCHAR(20) DEFAULT 'cloud',
    status VARCHAR(20) DEFAULT 'pending',
    file_count INTEGER DEFAULT 0,
    cloud_link VARCHAR(255),
    tracking_number VARCHAR(100),
    note TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- 自定义预约表单表
CREATE TABLE appointment_forms (
    id SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES creator_profiles(id),
    title VARCHAR(255),
    description TEXT,
    form_config JSONB,
    usage_count INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 表单字段表
CREATE TABLE form_fields (
    id SERIAL PRIMARY KEY,
    form_id INTEGER REFERENCES appointment_forms(id),
    field_name VARCHAR(100),
    field_type VARCHAR(20),
    label VARCHAR(255),
    placeholder VARCHAR(255),
    options JSONB,
    validation_rules JSONB,
    is_required BOOLEAN DEFAULT FALSE,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 审核表
CREATE TABLE audits (
    id SERIAL PRIMARY KEY,
    entity_type VARCHAR(50),
    entity_id INTEGER,
    status VARCHAR(20) DEFAULT 'pending',
    reviewer_id INTEGER,
    review_note TEXT,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 报表表
CREATE TABLE reports (
    id SERIAL PRIMARY KEY,
    report_type VARCHAR(50),
    status VARCHAR(20) DEFAULT 'pending',
    parameters JSONB,
    result JSONB,
    created_by INTEGER,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_events_aggregate ON domain_events(aggregate_type, aggregate_id);
CREATE INDEX idx_events_type ON domain_events(event_type);
CREATE INDEX idx_events_created ON domain_events(created_at);
CREATE INDEX idx_appointments_creator ON appointments(creator_id, appointment_date);
CREATE INDEX idx_appointments_user ON appointments(user_id);
CREATE INDEX idx_appointments_status ON appointments(status);
CREATE INDEX idx_services_type ON services(type_id);
CREATE INDEX idx_services_price ON services(price);
CREATE INDEX idx_services_creator ON services(creator_id);
CREATE INDEX idx_services_active ON services(is_active, is_featured);
CREATE INDEX idx_portfolios_creator ON portfolios(creator_id);
CREATE INDEX idx_portfolios_category ON portfolios(category);
CREATE INDEX idx_works_portfolio ON works(portfolio_id);
CREATE INDEX idx_reviews_creator ON reviews(creator_id);
CREATE INDEX idx_reviews_appointment ON reviews(appointment_id);
CREATE INDEX idx_notifications_user ON notifications(user_id, is_read);
CREATE INDEX idx_favorites_user ON favorites(user_id, target_type);
CREATE INDEX idx_prepaids_user ON prepaids(user_id);
CREATE INDEX idx_withdrawals_creator ON withdrawals(creator_id);
CREATE INDEX idx_withdrawals_status ON withdrawals(status);
```

### 6.2 ClickHouse 分析表

```sql
-- 预约事件流
CREATE TABLE appointment_events (
    event_type String,
    user_id UInt32,
    creator_id UInt32,
    service_type String,
    price Decimal(10,2),
    city String,
    event_time DateTime
) ENGINE = MergeTree()
ORDER BY (event_time, event_type);

-- 用户行为流
CREATE TABLE user_actions (
    user_id UInt32,
    action String,
    target_type String,
    target_id UInt32,
    action_time DateTime
) ENGINE = MergeTree()
ORDER BY (action_time, user_id);
```

---

## 七、项目结构

```
photography-ai-platform/
│
├── CLAUDE.md
├── Makefile
├── docker-compose.yml
├── docker-compose.prod.yml
├── .env.example
├── .github/workflows/
│   ├── ci.yml
│   └── deploy.yml
│
├── gateway/
│   ├── Dockerfile
│   ├── nginx.conf
│   └── conf.d/
│       ├── upstream.conf
│       ├── gateway.conf
│       └── static.conf
│
├── backend/
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── migrations/
│   ├── crates/
│   │   ├── core/
│   │   │   └── src/
│   │   │       ├── models/
│   │   │       │   ├── user.rs
│   │   │       │   ├── creator.rs
│   │   │       │   ├── service.rs
│   │   │       │   ├── appointment.rs
│   │   │       │   ├── payment.rs
│   │   │       │   ├── portfolio.rs
│   │   │       │   ├── review.rs
│   │   │       │   ├── notification.rs
│   │   │       │   ├── delivery.rs
│   │   │       │   ├── event.rs
│   │   │       │   └── mod.rs
│   │   │       ├── events/
│   │   │       │   ├── appointment_events.rs
│   │   │       │   ├── payment_events.rs
│   │   │       │   ├── user_events.rs
│   │   │       │   └── mod.rs
│   │   │       └── value_objects/
│   │   │           ├── money.rs
│   │   │           ├── time_slot.rs
│   │   │           ├── email.rs
│   │   │           ├── phone.rs
│   │   │           └── mod.rs
│   │   │
│   │   ├── api/
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── state.rs
│   │   │       ├── error.rs
│   │   │       ├── routes/
│   │   │       │   ├── auth.rs
│   │   │       │   ├── users.rs
│   │   │       │   ├── creators.rs
│   │   │       │   ├── services.rs
│   │   │       │   ├── appointments.rs
│   │   │       │   ├── payments.rs
│   │   │       │   ├── portfolios.rs
│   │   │       │   ├── reviews.rs
│   │   │       │   ├── notifications.rs
│   │   │       │   ├── deliveries.rs
│   │   │       │   ├── favorites.rs
│   │   │       │   ├── announcements.rs
│   │   │       │   ├── admin.rs
│   │   │       │   ├── ai.rs
│   │   │       │   ├── analytics.rs
│   │   │       │   ├── search.rs
│   │   │       │   ├── ws.rs
│   │   │       │   ├── health.rs
│   │   │       │   └── mod.rs
│   │   │       ├── middleware/
│   │   │       │   ├── auth.rs
│   │   │       │   ├── rate_limit.rs
│   │   │       │   ├── logging.rs
│   │   │       │   ├── request_id.rs
│   │   │       │   ├── cors.rs
│   │   │       │   └── mod.rs
│   │   │       └── dto/
│   │   │           ├── response.rs
│   │   │           ├── pagination.rs
│   │   │           ├── auth_dto.rs
│   │   │           ├── appointment_dto.rs
│   │   │           ├── service_dto.rs
│   │   │           ├── portfolio_dto.rs
│   │   │           ├── user_dto.rs
│   │   │           └── mod.rs
│   │   │
│   │   ├── services/
│   │   │   └── src/
│   │   │       ├── auth_service.rs
│   │   │       ├── user_service.rs
│   │   │       ├── appointment_service.rs
│   │   │       ├── payment_service.rs
│   │   │       ├── portfolio_service.rs
│   │   │       ├── review_service.rs
│   │   │       ├── notification_service.rs
│   │   │       ├── delivery_service.rs
│   │   │       ├── favorite_service.rs
│   │   │       ├── announcement_service.rs
│   │   │       ├── event_service.rs
│   │   │       ├── analytics_service.rs
│   │   │       ├── search_service.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── repository/
│   │   │   └── src/
│   │   │       ├── user_repo.rs
│   │   │       ├── appointment_repo.rs
│   │   │       ├── event_store.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── task_queue/
│   │   │   └── src/
│   │   │       ├── producer.rs
│   │   │       ├── consumer.rs
│   │   │       ├── handlers/
│   │   │       │   ├── email_handler.rs
│   │   │       │   ├── ai_handler.rs
│   │   │       │   ├── analytics_handler.rs
│   │   │       │   └── mod.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── cache/
│   │   │   └── src/
│   │   │       ├── redis.rs
│   │   │       ├── distributed_lock.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── search/
│   │   │   └── src/
│   │   │       ├── meilisearch.rs
│   │   │       ├── vector_store.rs
│   │   │       ├── hybrid.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── storage/
│   │   │   └── src/
│   │   │       ├── minio.rs
│   │   │       └── mod.rs
│   │   │
│   │   ├── ai_client/
│   │   │   ├── build.rs
│   │   │   ├── proto/ai_service.proto
│   │   │   └── src/
│   │   │       ├── client.rs
│   │   │       └── mod.rs
│   │   │
│   │   └── common/
│   │       └── src/
│   │           ├── config.rs
│   │           ├── error.rs
│   │           ├── telemetry.rs
│   │           └── mod.rs
│   │
│   ├── tests/
│   └── benches/
│
├── ai_service/
│   ├── Dockerfile
│   ├── pyproject.toml
│   ├── proto/ai_service.proto
│   ├── server.py
│   ├── inference/
│   │   ├── style_analyzer.py
│   │   ├── quality_assessor.py
│   │   ├── image_enhancer.py
│   │   ├── background_remover.py
│   │   └── model_manager.py
│   ├── agent/
│   │   ├── chat_agent.py
│   │   ├── tools.py
│   │   ├── prompts.py
│   │   └── memory.py
│   ├── search/
│   │   ├── embedding.py
│   │   └── reranker.py
│   └── content/
│       ├── copywriter.py
│       └── tag_generator.py
│
├── frontend/
│   ├── Dockerfile
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── uno.config.ts
│   └── src/
│       ├── index.html
│       ├── index.tsx
│       ├── app.tsx
│       ├── api/generated/
│       ├── components/
│       │   ├── common/
│       │   ├── layout/
│       │   ├── ai/
│       │   │   ├── ChatAssistant.tsx
│       │   │   ├── StyleMatcher.tsx
│       │   │   ├── SmartSearch.tsx
│       │   │   └── PhotoEditor.tsx
│       │   └── charts/
│       ├── composables/
│       │   ├── createAuth.ts
│       │   ├── createWebSocket.ts
│       │   ├── createAIChat.ts
│       │   ├── createSmartSearch.ts
│       │   ├── createPagination.ts
│       │   └── createNotification.ts
│       ├── pages/
│       │   ├── public/
│       │   │   ├── Landing.tsx
│       │   │   ├── Login.tsx
│       │   │   ├── Register.tsx
│       │   │   ├── ServiceList.tsx
│       │   │   ├── ServiceDetail.tsx
│       │   │   ├── CreatorList.tsx
│       │   │   ├── CreatorDetail.tsx
│       │   │   ├── Portfolios.tsx
│       │   │   ├── PortfolioDetail.tsx
│       │   │   ├── AIProcessor.tsx
│       │   │   ├── Agreement.tsx
│       │   │   └── Privacy.tsx
│       │   ├── user/
│       │   │   ├── Dashboard.tsx
│       │   │   ├── Orders.tsx
│       │   │   ├── Favorites.tsx
│       │   │   ├── Reviews.tsx
│       │   │   ├── Notifications.tsx
│       │   │   ├── AIHelper.tsx
│       │   │   └── Certification.tsx
│       │   ├── creator/
│       │   │   ├── Dashboard.tsx
│       │   │   ├── Orders.tsx
│       │   │   ├── Services.tsx
│       │   │   ├── Portfolios.tsx
│       │   │   ├── Works.tsx
│       │   │   ├── Analytics.tsx
│       │   │   └── Earnings.tsx
│       │   └── admin/
│       │       ├── Dashboard.tsx
│       │       ├── Users.tsx
│       │       ├── Creators.tsx
│       │       ├── Appointments.tsx
│       │       ├── Payments.tsx
│       │       ├── Services.tsx
│       │       ├── Reviews.tsx
│       │       ├── Announcements.tsx
│       │       ├── Certifications.tsx
│       │       ├── Analytics.tsx
│       │       └── Settings.tsx
│       ├── stores/
│       │   ├── auth.ts
│       │   ├── theme.ts
│       │   └── ws.ts
│       ├── router/
│       ├── utils/
│       └── styles/
│
├── monitoring/
│   ├── prometheus/
│   │   └── prometheus.yml
│   ├── grafana/
│   │   ├── provisioning/
│   │   └── dashboards/
│   │       ├── api-overview.json
│   │       ├── ai-inference.json
│   │       ├── business-metrics.json
│   │       └── infrastructure.json
│   └── loki/
│       └── loki.yml
│
├── scripts/
│   ├── seed.py
│   ├── migrate.sh
│   ├── deploy.sh
│   └── generate-api.sh
│
└── docs/
    ├── architecture.md
    ├── api.md
    └── deployment.md
```

---

## 八、核心代码模式

### 8.1 Rust — 应用入口

```rust
// crates/api/src/main.rs
use axum::{Router, middleware};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

mod routes;
mod middleware as app_middleware;
mod state;
mod error;

use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化遥测
    common::telemetry::init()?;

    // 构建应用状态（DI 容器）
    let state = AppState::new().await?;

    // 路由
    let app = Router::new()
        .nest("/api/v1", routes::api_routes())
        .layer(middleware::from_fn(app_middleware::logging))
        .layer(middleware::from_fn(app_middleware::auth))
        .layer(app_middleware::rate_limit())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // 启动
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    tracing::info!("Server running on :8000");
    axum::serve(listener, app).await?;

    Ok(())
}
```

### 8.2 Rust — 统一响应

```rust
// crates/api/src/dto/response.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub has_next: bool,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { code: 200, message: "success".into(), data: Some(data) }
    }
}

impl ApiResponse<()> {
    pub fn error(code: u16, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), data: None }
    }
}
```

### 8.3 Rust — 统一错误

```rust
// crates/api/src/error.rs
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("权限不足")]
    Forbidden,
    #[error("未认证")]
    Unauthorized,
    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = match &self {
            AppError::NotFound(_)   => (StatusCode::NOT_FOUND, 404),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, 400),
            AppError::Conflict(_)   => (StatusCode::CONFLICT, 409),
            AppError::Forbidden     => (StatusCode::FORBIDDEN, 403),
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED, 401),
            AppError::Internal(_)   => (StatusCode::INTERNAL_SERVER_ERROR, 500),
        };
        (status, Json(json!({ "code": code, "message": self.to_string() }))).into_response()
    }
}
```

### 8.4 Rust — 认证中间件

```rust
// crates/api/src/middleware/auth.rs
use axum::{extract::Request, middleware::Next, response::Response};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Clone)]
pub struct CurrentUser {
    pub user_id: i32,
    pub username: String,
    pub roles: Vec<String>,
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, AppError> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;

    let claims = decode::<Claims>(token, &KEY, &VALIDATION)
        .map_err(|_| AppError::Unauthorized)?;

    req.extensions_mut().insert(CurrentUser {
        user_id: claims.sub,
        username: claims.username,
        roles: claims.roles,
    });

    Ok(next.run(req).await)
}
```

### 8.5 Rust — 路由处理

```rust
// crates/api/src/routes/appointments.rs
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create).get(list))
        .route("/:id", get(detail).put(update))
        .route("/:id/confirm", put(confirm))
        .route("/:id/cancel", put(cancel))
}

async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<CreateAppointmentDto>,
) -> Result<ApiResponse<AppointmentDto>, AppError> {
    let result = state.appointment_service.create(user.user_id, body).await?;
    Ok(ApiResponse::success(result.into()))
}

async fn detail(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i32>,
) -> Result<ApiResponse<AppointmentDetailDto>, AppError> {
    let result = state.appointment_service
        .get_by_id(id, user.user_id, &user.roles)
        .await?
        .ok_or_else(|| AppError::NotFound("预约不存在".into()))?;
    Ok(ApiResponse::success(result.into()))
}
```

### 8.6 Rust — 值对象

```rust
// crates/core/src/value_objects/money.rs
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Money(Decimal);

#[derive(Debug, Error)]
pub enum MoneyError {
    #[error("金额不能为负数")]
    Negative,
    #[error("金额精度不能超过2位小数")]
    TooManyDecimals,
}

impl Money {
    pub fn new(amount: Decimal) -> Result<Self, MoneyError> {
        if amount < Decimal::ZERO {
            return Err(MoneyError::Negative);
        }
        if amount.scale() > 2 {
            return Err(MoneyError::TooManyDecimals);
        }
        Ok(Self(amount))
    }

    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    pub fn amount(&self) -> &Decimal {
        &self.0
    }
}

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

// crates/core/src/value_objects/time_slot.rs
use chrono::{DateTime, Utc, Duration};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct TimeSlot {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum TimeSlotError {
    #[error("结束时间必须在开始时间之后")]
    EndBeforeStart,
    #[error("时段长度不能小于1小时")]
    TooShort,
    #[error("时段长度不能超过8小时")]
    TooLong,
    #[error("不能预约过去的时段")]
    InThePast,
}

impl TimeSlot {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Self, TimeSlotError> {
        if end <= start {
            return Err(TimeSlotError::EndBeforeStart);
        }
        let duration = end - start;
        if duration < Duration::hours(1) {
            return Err(TimeSlotError::TooShort);
        }
        if duration > Duration::hours(8) {
            return Err(TimeSlotError::TooLong);
        }
        if start < Utc::now() {
            return Err(TimeSlotError::InThePast);
        }
        Ok(Self { start, end })
    }

    pub fn overlaps(&self, other: &TimeSlot) -> bool {
        self.start < other.end && other.start < self.end
    }
}
```

### 8.7 Rust — 分布式锁

```rust
// crates/cache/src/distributed_lock.rs
use redis::AsyncCommands;
use std::time::Duration;

pub struct DistributedLock {
    redis: redis::aio::ConnectionManager,
}

impl DistributedLock {
    pub async fn acquire(&self, key: &str, timeout: Duration) -> Result<LockGuard, LockError> {
        let mut conn = self.redis.clone();
        let lock_value = uuid::Uuid::new_v4().to_string();
        let acquired: bool = conn.set_nx(key, &lock_value).await?;

        if !acquired {
            return Err(LockError::AlreadyLocked);
        }

        conn.expire(key, timeout.as_secs() as i64).await?;

        Ok(LockGuard {
            redis: self.redis.clone(),
            key: key.to_string(),
            value: lock_value,
        })
    }
}

pub struct LockGuard {
    redis: redis::aio::ConnectionManager,
    key: String,
    value: String,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        let mut conn = self.redis.clone();
        let key = self.key.clone();
        let value = self.value.clone();
        tokio::spawn(async move {
            let _: Result<(), _> = redis::cmd("EVAL")
                .arg("if redis.call('get',KEYS[1])==ARGV[1] then return redis.call('del',KEYS[1]) else return 0 end")
                .arg(1)
                .arg(&key)
                .arg(&value)
                .query_async(&mut conn)
                .await;
        });
    }
}
```

### 8.8 Rust — 事件溯源

```rust
// crates/repository/src/event_store.rs
use sea_orm::{EntityTrait, ActiveModelTrait, QueryOrder};

pub struct EventStore {
    db: DatabaseConnection,
    redis: redis::aio::ConnectionManager,
}

impl EventStore {
    pub async fn append(
        &self,
        aggregate_type: &str,
        aggregate_id: i32,
        event_type: &str,
        event_data: serde_json::Value,
        created_by: i32,
    ) -> Result<DomainEventModel, AppError> {
        let last = DomainEventEntity::find()
            .filter(domain_events::Column::AggregateType.eq(aggregate_type))
            .filter(domain_events::Column::AggregateId.eq(aggregate_id))
            .order_by_desc(domain_events::Column::Version)
            .one(&self.db)
            .await?;

        let version = last.map(|e| e.version + 1).unwrap_or(1);

        let event = domain_events::ActiveModel {
            aggregate_type: Set(aggregate_type.to_string()),
            aggregate_id: Set(aggregate_id),
            event_type: Set(event_type.to_string()),
            event_data: Set(event_data),
            version: Set(version),
            created_by: Set(created_by),
            ..Default::default()
        };

        let model = event.insert(&self.db).await?;

        let channel = format!("events:{}:{}", aggregate_type, aggregate_id);
        let _ = self.redis.publish::<_, _, ()>(&channel, serde_json::to_string(&model)?).await;

        Ok(model)
    }

    pub async fn get_events(
        &self,
        aggregate_type: &str,
        aggregate_id: i32,
    ) -> Result<Vec<DomainEventModel>, AppError> {
        Ok(DomainEventEntity::find()
            .filter(domain_events::Column::AggregateType.eq(aggregate_type))
            .filter(domain_events::Column::AggregateId.eq(aggregate_id))
            .order_by_asc(domain_events::Column::Version)
            .all(&self.db)
            .await?)
    }
}
```

### 8.9 SolidJS — 认证状态

```tsx
// frontend/src/stores/auth.ts
import { createSignal } from "solid-js";

interface User {
  id: number;
  username: string;
  role: "admin" | "creator" | "user";
  avatar?: string;
}

const [user, setUser] = createSignal<User | null>(null);
const [token, setToken] = createSignal(localStorage.getItem("token") || "");

export const auth = {
  user, token,
  isLogin: () => !!token(),

  async login(username: string, password: string) {
    const res = await api.post("/auth/login", { username, password });
    const t = res.data.access_token;
    setToken(t);
    localStorage.setItem("token", t);
    await this.fetchUser();
    return res;
  },

  async fetchUser() {
    const res = await api.get("/auth/profile");
    setUser({
      id: res.data.user_id,
      username: res.data.username,
      role: res.data.roles?.[0] || "user",
      avatar: res.data.avatar_url,
    });
  },

  logout() {
    setToken("");
    setUser(null);
    localStorage.removeItem("token");
  },
};
```

### 8.10 SolidJS — WebSocket

```tsx
// frontend/src/composables/createWebSocket.ts
import { createSignal, onCleanup } from "solid-js";

export function createWebSocket(url: string) {
  const [messages, setMessages] = createSignal<any[]>([]);
  const [status, setStatus] = createSignal<"connecting" | "open" | "closed">("connecting");

  const ws = new WebSocket(url);

  ws.onopen = () => setStatus("open");
  ws.onclose = () => setStatus("closed");
  ws.onmessage = (e) => {
    const data = JSON.parse(e.data);
    setMessages((prev) => [...prev, data]);
  };

  onCleanup(() => ws.close());

  return {
    messages,
    status,
    send: (data: any) => ws.send(JSON.stringify(data)),
  };
}
```

### 8.11 SolidJS — AI 对话

```tsx
// frontend/src/components/ai/ChatAssistant.tsx
import { createSignal, For, Show } from "solid-js";
import { createAIChat } from "../../composables/createAIChat";

export function ChatAssistant() {
  const { messages, loading, send } = createAIChat();
  const [input, setInput] = createSignal("");

  return (
    <div class="flex flex-col h-full">
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <For each={messages()}>
          {(msg) => (
            <div class={`flex ${msg.role === "user" ? "justify-end" : "justify-start"}`}>
              <div class={`max-w-[80%] rounded-lg px-4 py-2 ${
                msg.role === "user" ? "bg-blue-500 text-white" : "bg-gray-100"
              }`}>{msg.content}</div>
            </div>
          )}
        </For>
        <Show when={loading()}>
          <div class="text-gray-400 animate-pulse">AI 思考中...</div>
        </Show>
      </div>
      <div class="p-4 border-t">
        <input
          class="w-full border rounded-lg px-4 py-2"
          placeholder="描述你想要的拍摄..."
          value={input()}
          onInput={(e) => setInput(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              send(input());
              setInput("");
            }
          }}
        />
      </div>
    </div>
  );
}
```

### 8.12 Python AI gRPC 服务

```python
# ai_service/server.py
import grpc
from concurrent import futures
from proto import ai_service_pb2 as pb2
from proto import ai_service_pb2_grpc as pb2_grpc
from inference.style_analyzer import StyleAnalyzer
from inference.quality_assessor import QualityAssessor
from inference.image_enhancer import ImageEnhancer
from agent.chat_agent import ChatAgent

class AIService(pb2_grpc.AIServiceServicer):
    def __init__(self):
        self.style = StyleAnalyzer()
        self.quality = QualityAssessor()
        self.enhancer = ImageEnhancer()
        self.agent = ChatAgent()

    def AnalyzeStyle(self, request, context):
        vector, labels = self.style.analyze(request.image_data)
        return pb2.StyleResponse(vector=vector, labels=labels)

    def AssessQuality(self, request, context):
        scores = self.quality.assess(request.image_data)
        return pb2.QualityResponse(**scores)

    def EnhanceImage(self, request, context):
        result = self.enhancer.enhance(request.image_data, request.mode)
        return pb2.ImageResponse(image_data=result)

    def Chat(self, request, context):
        for chunk in self.agent.stream(request.user_id, request.message):
            yield pb2.ChatChunk(content=chunk)

def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=4))
    pb2_grpc.add_AIServiceServicer_to_server(AIService(), server)
    server.add_insecure_port("[::]:50051")
    server.start()
    print("AI Service running on :50051")
    server.wait_for_termination()

if __name__ == "__main__":
    serve()
```

### 8.13 Python AI Agent

```python
# ai_service/agent/chat_agent.py
from typing import AsyncIterator

class ChatAgent:
    def __init__(self):
        self.llm = ChatLLM(model="qwen2.5-72b")
        self.tools = self._build_tools()
        self.memory = ConversationMemory()

    def _build_tools(self):
        return [
            Tool("search_creators", "根据风格/预算/地点搜索创作者", self._search_creators),
            Tool("check_availability", "查询创作者可用时段", self._check_availability),
            Tool("create_appointment", "创建预约", self._create_appointment),
            Tool("estimate_price", "估算拍摄价格", self._estimate_price),
            Tool("get_style_recommendations", "根据需求推荐风格", self._get_style_recommendations),
            Tool("check_weather", "查询指定日期天气", self._check_weather),
            Tool("generate_mood_board", "生成灵感板", self._generate_mood_board),
        ]

    async def stream(self, user_id: int, message: str) -> AsyncIterator[str]:
        messages = await self.memory.load(user_id)
        messages.append({"role": "user", "content": message})

        while True:
            response = await self.llm.chat(messages, tools=self.tools)

            if response.tool_calls:
                for call in response.tool_calls:
                    result = await self.tools[call.name].execute(call.args)
                    messages.append({"role": "tool", "content": result})
                continue

            async for chunk in response.stream():
                yield chunk
            break

        await self.memory.save(user_id, messages)
```

---

## 九、Makefile

```makefile
.PHONY: dev dev-full build test lint fmt migrate generate-api deploy monitor clean

dev:
	docker compose up -d postgres redis qdrant meilisearch minio
	cd backend && cargo run -p api
	cd ai_service && python server.py &
	cd frontend && npm run dev

dev-full:
	docker compose up -d
	@echo "Frontend:   http://localhost:5173"
	@echo "API:        http://localhost:8000"
	@echo "API Docs:   http://localhost:8000/docs"
	@echo "Grafana:    http://localhost:3000"
	@echo "Prometheus: http://localhost:9090"
	@echo "MinIO:      http://localhost:9001"

build:
	cd backend && cargo build --release
	cd frontend && npm run build
	docker compose -f docker-compose.prod.yml build

test:
	cd backend && cargo test
	cd frontend && npm run test
	cd ai_service && pytest

bench:
	cd backend && cargo bench

lint:
	cd backend && cargo clippy -- -D warnings
	cd backend && cargo fmt --check
	cd frontend && npm run lint
	cd ai_service && ruff check .

fmt:
	cd backend && cargo fmt
	cd frontend && npm run format
	cd ai_service && ruff format .

migrate:
	cd backend && sea-orm-cli migrate up

migrate-new:
	cd backend && sea-orm-cli migrate generate $(name)

generate-api:
	cd frontend && npx openapi-typescript-codegen \
		--input http://localhost:8000/api/openapi.json \
		--output src/api/generated

deploy:
	docker compose -f docker-compose.prod.yml up -d --build

monitor:
	@echo "Grafana:    http://localhost:3000"
	@echo "Prometheus: http://localhost:9090"

clean:
	cd backend && cargo clean
	cd frontend && rm -rf node_modules dist
	docker compose down -v
```

---

## 十、Docker Compose

```yaml
services:
  nginx:
    build: ./gateway
    ports: ["80:80", "443:443"]
    depends_on: [backend, frontend]

  backend:
    build: { context: ./backend, dockerfile: Dockerfile }
    ports: ["8000:8000"]
    environment:
      DATABASE_URL: postgres://app:secret@postgres:5432/photography
      REDIS_URL: redis://redis:6379
      QDRANT_URL: http://qdrant:6333
      MEILISEARCH_URL: http://meilisearch:7700
      MEILISEARCH_KEY: master_key
      AI_SERVICE_URL: http://ai-service:50051
      JWT_SECRET: change_me_in_production
      RUST_LOG: info
    depends_on: [postgres, redis, qdrant, meilisearch]

  ai-service:
    build: { context: ./ai_service, dockerfile: Dockerfile }
    ports: ["50051:50051"]
    deploy:
      resources:
        reservations:
          devices: [{ capabilities: [gpu] }]

  frontend:
    build: ./frontend
    ports: ["5173:5173"]

  postgres:
    image: postgres:16-alpine
    environment: { POSTGRES_DB: photography, POSTGRES_USER: app, POSTGRES_PASSWORD: secret }
    volumes: [pgdata:/var/lib/postgresql/data]
    ports: ["5432:5432"]

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes: [redisdata:/data]

  qdrant:
    image: qdrant/qdrant:latest
    ports: ["6333:6333"]
    volumes: [qdrantdata:/qdrant/storage]

  meilisearch:
    image: getmeili/meilisearch:latest
    environment: { MEILI_MASTER_KEY: master_key }
    ports: ["7700:7700"]
    volumes: [msdata:/meili_data]

  clickhouse:
    image: clickhouse/clickhouse-server:latest
    ports: ["8123:8123"]

  minio:
    image: minio/minio
    command: server /data --console-address ":9001"
    ports: ["9000:9000", "9001:9001"]
    environment: { MINIO_ROOT_USER: minioadmin, MINIO_ROOT_PASSWORD: minioadmin }
    volumes: [miniodata:/data]

  prometheus:
    image: prom/prometheus
    volumes: ["./monitoring/prometheus:/etc/prometheus"]
    ports: ["9090:9090"]

  grafana:
    image: grafana/grafana
    ports: ["3000:3000"]
    volumes: ["./monitoring/grafana/provisioning:/etc/grafana/provisioning"]

  loki:
    image: grafana/loki
    ports: ["3100:3100"]

volumes:
  pgdata:
  redisdata:
  qdrantdata:
  msdata:
  miniodata:
```

---

## 十一、开发规范

### 11.1 Rust

- 错误处理：业务用 `thiserror`，系统用 `anyhow`
- 异步：全部 `async/await`，阻塞操作用 `tokio::task::spawn_blocking`
- 日志：`tracing::info!` / `tracing::error!`，带结构化字段
- 测试：单元测试 `#[cfg(test)]`，集成测试 `tests/`
- 命名：snake_case 函数/变量，PascalCase 类型
- 注释：只注释"为什么"

### 11.2 SolidJS

- 组件：PascalCase 文件名和组件名
- 状态：`createSignal` 管理局部状态，全局状态用 context
- 数据获取：`@tanstack/solid-query`，不用手动 fetch
- 样式：UnoCSS 原子类
- 类型：严格 TypeScript，不用 `any`

### 11.3 Python

- 格式：Black + Ruff
- 类型：全面 type hints
- 测试：pytest
- 日志：structlog

### 11.4 Git

- 分支：`main` (生产) / `develop` (开发) / `feature/*` / `fix/*`
- 提交：`feat:` / `fix:` / `refactor:` / `docs:` / `test:`
- PR：CI 必须通过，至少 1 人 review

---

## 十二、迁移策略

```
Phase 1: 骨架
  - Cargo Workspace 初始化
  - Axum + SeaORM + PostgreSQL 连通
  - 认证模块跑通
  - Docker Compose 启动
  - SolidJS 项目初始化 + 登录页

Phase 2: 核心业务
  - 预约模块 + 分布式锁
  - 支付模块
  - 服务模块
  - 事件溯源基础设施
  - 前端核心页面

Phase 3: AI 能力
  - Python AI 服务 + gRPC
  - 风格分析 + 质量评估
  - 智能搜索 (CLIP + Qdrant)
  - AI Agent 对话
  - 前端 AI 组件

Phase 4: 高级功能
  - ClickHouse 数据分析
  - 智能调度
  - 推荐系统
  - 管理后台数据看板

Phase 5: 生产化
  - 可观测性 (Prometheus + Grafana + Loki)
  - CI/CD (GitHub Actions)
  - 性能基准测试
  - 安全审计
  - 文档完善
```

---

## 十三、为什么这能成为教科书

| 维度 | 意义 |
|------|------|
| **Rust 做业务系统** | 市面几乎没有参考，你做的就是参考实现 |
| **SolidJS + Rust** | 前后端都追求零开销抽象，哲学一致 |
| **事件溯源落地** | 不是概念，是真正可用的生产级实现 |
| **值对象设计** | 编译期强制业务规则，Rust 版 DDD 教科书 |
| **AI 原生架构** | AI 不是附加功能，是核心能力层 |
| **Rust + Python 协同** | 系统语言 + AI 生态的最佳协作模式 |
| **全栈类型安全** | TypeScript ↔ Rust ↔ PostgreSQL 全链路 |
| **可观测性** | 三支柱完整落地，不是"加了日志" |
| **一键启动** | `make dev` 启动 12+ 个服务 |
| **模块化** | Cargo Workspace，每个 crate 职责单一 |

---

## 十四、超越时代：存储层进化

### 14.1 为什么 PostgreSQL 不够

PostgreSQL 是可靠的基础，但对于一个"超越时代"的项目，它只是起点：

| 问题 | PostgreSQL 的局限 | 进化方案 |
|------|-------------------|---------|
| 开发环境 | 手动创建测试数据库 | **Neon 数据库分支** — 像 git branch 一样管理数据库 |
| 图关系 | JOIN 深度有限 | **原生图遍历** — 用户-创作者-服务-作品的关系网络 |
| 实时性 | 需要外部 WebSocket | **内置实时订阅** — 数据变更自动推送到前端 |
| 向量搜索 | 需要 pgvector 扩展 | **原生向量存储** — 风格 DNA 是一等公民 |
| 全文搜索 | 中文分词弱 | **专用搜索引擎** — Meilisearch 的中文分词 |

### 14.2 Neon：数据库分支革命

Neon 是 Serverless PostgreSQL，但它的杀手特性是**数据库分支**：

```
main (生产数据库)
  ├── dev (开发分支，即时创建，零数据复制)
  ├── test (测试分支，CI 自动创建)
  ├── preview/PR-123 (PR 预览分支)
  └── staging (预发布分支)
```

**开发体验**：
- `git checkout feature/new-service` → 数据库自动切到对应分支
- PR 创建 → 自动创建预览数据库 → 前端可直接测试
- 测试完成 → 分支删除 → 零清理成本

**生产优势**：
- Scale-to-zero：无流量时自动休眠，零成本
- 即时恢复：Point-in-time recovery 到任意时间点
- 读写分离：读副本自动扩展

### 14.3 智能合约：信任最小化

用 Rust 的 `ink!` 在 Polkadot 上实现去中心化功能：

#### 14.3.1 托管支付合约

```
客户支付 → 资金锁定在合约中
                ↓
        创作者完成服务
                ↓
    客户确认 → 资金自动释放给创作者
        或
    超时/争议 → 仲裁机制退款
```

**传统模式**：平台托管资金 → 需要信任平台
**智能合约**：代码托管资金 → 信任数学，不信任人

```rust
// ink! 智能合约伪代码
#[ink(contract)]
pub mod escrow {
    #[ink(storage)]
    pub struct Escrow {
        payments: Mapping<AppointmentId, Payment>,
    }

    #[ink(message)]
    pub fn create_escrow(&mut self, appointment_id: AppointmentId, seller: AccountId) {
        let payment = Payment {
            buyer: self.env().caller(),
            seller,
            amount: self.env().transferred_value(),
            status: EscrowStatus::Locked,
        };
        self.payments.insert(appointment_id, &payment);
    }

    #[ink(message)]
    pub fn release(&mut self, appointment_id: AppointmentId) {
        let payment = self.payments.get(appointment_id).unwrap();
        assert_eq!(self.env().caller(), payment.buyer);
        self.env().transfer(payment.seller, payment.amount).unwrap();
    }
}
```

#### 14.3.2 创作者认证凭证

```
创作者通过审核 → 链上颁发可验证凭证
                ↓
    凭证包含：认证等级、技能标签、评分
                ↓
    任何人可验证：调用合约 verify(creator_address)
                ↓
    凭证不可伪造、不可篡改、可跨平台携带
```

#### 14.3.3 版税分配合约

```
作品被使用/转载 → 智能合约自动分配版税
                ↓
    原创作者: 70% | 平台: 15% | 推荐人: 15%
                ↓
    链上透明、自动执行、无需人工结算
```

### 14.4 CRDT：无冲突实时协作

用 Yrs（Rust 版 Yjs CRDT 框架）实现协作功能：

#### 14.4.1 共享灵感板

```
客户和创作者同时编辑灵感板
        ↓
CRDT 保证：无论操作顺序如何，最终状态一致
        ↓
不需要锁定、不需要合并冲突、不需要中央协调
```

#### 14.4.2 实时聊天增强

```
传统聊天：消息可能乱序、可能丢失
CRDT 聊天：消息自动排序、自动合并、离线同步
```

#### 14.4.3 协作文档

```
创作者和客户共同编辑拍摄方案
        ↓
实时看到对方的光标和编辑
        ↓
离线编辑 → 上线后自动同步 → 无冲突
```

### 14.5 向量原生：风格 DNA 系统

不是"数据库里加了个向量字段"，是**整个风格系统以向量为核心**：

```
┌─────────────────────────────────────────────────┐
│                 风格 DNA 系统                      │
├─────────────────────────────────────────────────┤
│                                                   │
│  图片上传 → CLIP ViT-L/14 → 768维向量             │
│                                                   │
│  向量存储：Qdrant (专用向量数据库)                  │
│  ├── 每张作品: 1个向量 + 风格标签                  │
│  ├── 每个作品集: 聚合向量 (所有作品的质心)         │
│  └── 每个创作者: 聚合向量 (所有作品的质心)         │
│                                                   │
│  查询方式:                                         │
│  ├── 文本 → CLIP编码 → 向量 → 最近邻              │
│  ├── 图片 → CLIP编码 → 向量 → 最近邻              │
│  └── 混合 → 文本+图片+筛选 → 多路召回+重排        │
│                                                   │
│  应用场景:                                         │
│  ├── "日系小清新" → 匹配风格相似的创作者           │
│  ├── 上传参考图 → 找到风格最接近的作品             │
│  └── "浪漫海景婚纱" → 融合语义+视觉的智能搜索     │
│                                                   │
└─────────────────────────────────────────────────┘
```

### 14.6 存储层最终选型

| 存储 | 用途 | 特性 |
|------|------|------|
| **Neon PostgreSQL** | 核心业务数据 | 分支、Serverless、ACID |
| **Qdrant** | 风格向量检索 | 768维向量、实时索引 |
| **Meilisearch** | 全文搜索 | 中文分词、typo-tolerant |
| **Redis** | 缓存/实时/队列 | Pub/Sub、Streams、分布式锁 |
| **ClickHouse** | OLAP 分析 | 列式存储、亿级秒查 |
| **MinIO** | 对象存储 | S3 兼容、图片/视频 |
| **Yrs CRDT** | 实时协作 | 无冲突复制、离线同步 |
| **ink! 智能合约** | 去中心化功能 | 托管支付、认证凭证、版税 |

---

## 十五、超越时代：架构哲学

### 15.1 零开销抽象

Rust 和 SolidJS 的共同信仰：**只做必要的事**。

- Rust：编译期内存管理，运行时零开销
- SolidJS：编译期依赖追踪，DOM 更新精确到字节
- 事件溯源：append-only 写入，物化视图异步构建
- 向量检索：近似最近邻，O(log n) 而非 O(n)

### 15.2 信任最小化

- 智能合约托管：信任代码，不信任平台
- 事件溯源：信任事件流，不信任当前状态
- 链上凭证：信任密码学，不信任人工审核

### 15.3 离线优先

- CRDT 协作：离线编辑，在线同步
- Service Worker：前端资源缓存
- 本地状态：localStorage + IndexedDB

### 15.4 渐进增强

- 核心功能不依赖 AI — 传统搜索/筛选作为降级方案
- 智能合约可选 — 传统支付作为默认方案
- 实时协作可选 — 传统编辑作为降级方案

### 15.5 为什么这能成为教科书

| 维度 | 意义 |
|------|------|
| **Rust 做业务系统** | 市面几乎没有参考，你做的就是参考实现 |
| **SolidJS + Rust** | 前后端都追求零开销抽象，哲学一致 |
| **事件溯源落地** | 不是概念，是真正可用的生产级实现 |
| **值对象设计** | 编译期强制业务规则，Rust 版 DDD 教科书 |
| **AI 原生架构** | AI 不是附加功能，是核心能力层 |
| **Rust + Python 协同** | 系统语言 + AI 生态的最佳协作模式 |
| **全栈类型安全** | TypeScript ↔ Rust ↔ PostgreSQL 全链路 |
| **智能合约集成** | 去中心化支付托管 + 链上凭证 |
| **CRDT 实时协作** | 无冲突复制，真正的协作体验 |
| **Neon 数据库分支** | 开发/测试/生产环境一键切换 |
| **向量原生搜索** | 风格 DNA 不是功能，是系统基因 |
| **可观测性** | 三支柱完整落地，不是"加了日志" |
| **一键启动** | `make dev` 启动 12+ 个服务 |
| **模块化** | Cargo Workspace，每个 crate 职责单一 |
