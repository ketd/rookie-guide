# 阅历进度条 (Rookie Guide) - V0.0.1

> 帮助年轻人从容应对"人生第一次"的生活成长伴侣

## 📖 项目简介

《阅历进度条》是一款通过社区共创的、可执行的、本地化的指南，将未知的人生挑战转化为清晰的成长路径的应用。

### 核心功能

- **阅历模板**: 浏览和搜索高质量的本地化生活指南
- **Fork 功能**: 将模板复制到个人空间，创建个人清单
- **进度追踪**: 勾选完成步骤，可视化展示完成进度
- **地理标签**: 支持通用和城市级别的本地化内容

## 🏗️ 技术架构

### 技术栈

- **语言**: Rust 2021 Edition
- **Web 框架**: Axum 0.7
- **ORM框架**: SeaORM 1.1（**实体模型 + 类型安全查询**）
- **数据库迁移**: SeaORM Migration（**Rust 代码定义迁移，类型安全**）
- **依赖注入**: 手动 DI 模式（基于 trait + Arc）
- **认证**: JWT + bcrypt
- **异步运行时**: Tokio

### ⚙️ 数据库管理

本项目使用 **SeaORM** 全栈解决方案：

✅ **SeaORM 实体模型** - 使用 Rust 类型定义数据库表结构  
✅ **类型安全查询** - 编译时检查查询，避免运行时错误  
✅ **SeaORM Migration** - Rust 代码定义迁移，编译时验证  
✅ **自动迁移执行** - 启动时自动运行所有pending迁移  
✅ **关系映射** - 支持一对多、多对一等关系定义  
✅ **JSON 字段支持** - 原生支持 PostgreSQL JSONB 类型  

详见 → [数据库管理文档](DATABASE.md)

### 多 Crate 架构

```
rookie-guide/
├── crates/
│   ├── api/          # Web API 服务层
│   ├── service_layer/# 核心业务逻辑（Service 层）
│   ├── models/       # 数据模型和 DTO（SeaORM 实体）
│   ├── db/           # 数据库访问层（Repository 层）
│   ├── migration/    # 数据库迁移（SeaORM Migration）
│   ├── auth/         # 用户认证模块
│   └── common/       # 公共工具库
└── Cargo.toml       # Workspace 配置
```

### 分层架构

```
API Layer (handlers)
      ↓
Service Layer (core)
      ↓
Repository Layer (db)
      ↓
Database (PostgreSQL)
```

## 🚀 快速开始

### 前置要求

- Rust 1.75+ 
- PostgreSQL 14+
- SQLx CLI (用于数据库迁移)

### 安装 SQLx CLI

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 快速启动

#### 🎯 一键启动（最简单）

```bash
# 复制配置文件
cp .env.example .env

# 一键完成：启动数据库 + 创建数据库 + 运行迁移 + 启动服务器
make init
```

#### 📝 分步启动

**1️⃣ 确保 PostgreSQL 服务运行**

```bash
# 方式A：使用 Docker（推荐）
docker-compose up -d

# 方式B：使用本地 PostgreSQL
# Windows: net start postgresql-x64-14
# macOS/Linux: sudo systemctl start postgresql
```

**2️⃣ 配置并启动**

```bash
# 复制并编辑配置文件
cp .env.example .env

# 编辑 .env，设置数据库连接信息
# DATABASE_HOST=localhost
# DATABASE_PASSWORD=password

# 启动项目（自动完成所有初始化）
cargo run -p api
# 或
make dev
```

**✨ 全自动初始化**：应用启动时会自动：
- 📦 **创建数据库**（如果不存在）
- 🔄 **运行数据库迁移**（创建表结构）
- 🚀 **启动 API 服务器**

**不需要手动执行 `createdb` 或 `sqlx migrate run`！**

服务将在 `http://127.0.0.1:8080` 启动。

### 健康检查

```bash
curl http://127.0.0.1:8080/health
```

## 📚 API 文档

### 认证

#### 注册
```http
POST /api/auth/register
Content-Type: application/json

{
  "phone": "13800138000",
  "email": "user@example.com",
  "password": "password123",
  "nickname": "张三"
}
```

#### 登录
```http
POST /api/auth/login
Content-Type: application/json

{
  "phone": "13800138000",
  "password": "password123"
}
```

### 用户

#### 获取当前用户信息
```http
GET /api/users/me
Authorization: Bearer <token>
```

#### 更新个人资料
```http
PUT /api/users/me
Authorization: Bearer <token>
Content-Type: application/json

{
  "nickname": "李四",
  "home_city": "CN-BJ"
}
```

### 模板

#### 列出所有模板
```http
GET /api/templates?page=1&page_size=20
```

#### 搜索模板
```http
GET /api/templates/search?keyword=租房&location_tag=CN-BJ
```

#### 获取单个模板
```http
GET /api/templates/:id
```

#### 创建模板
```http
POST /api/templates
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "第一次在北京租房整租指南",
  "description": "详细的北京租房步骤清单",
  "location_tag": "CN-BJ",
  "steps": [
    {
      "title": "确定预算和区域",
      "description": "根据工作地点和收入确定租房预算",
      "order": 0
    },
    {
      "title": "寻找房源",
      "description": "通过正规渠道寻找房源",
      "order": 1
    }
  ]
}
```

### 清单

#### Fork 模板到个人清单
```http
POST /api/checklists
Authorization: Bearer <token>
Content-Type: application/json

{
  "template_id": "uuid-here"
}
```

#### 获取用户的所有清单
```http
GET /api/checklists
Authorization: Bearer <token>
```

#### 获取单个清单详情
```http
GET /api/checklists/:id
Authorization: Bearer <token>
```

#### 更新步骤状态
```http
PUT /api/checklists/:id/steps
Authorization: Bearer <token>
Content-Type: application/json

{
  "step_index": 0,
  "completed": true
}
```

## 🔧 依赖注入设计

本项目使用手动依赖注入模式，具有以下特点：

1. **Repository 层**: 通过 trait 定义接口，实现数据访问抽象
2. **Service 层**: 依赖 Repository trait，通过构造函数注入
3. **AppModule**: 集中管理所有依赖的创建和注入
4. **Arc<dyn Trait>**: 使用智能指针实现运行时多态

### DI 流程

```rust
// 1. 创建数据库连接池
let pool = create_pool(&db_url, max_conn).await?;

// 2. 创建 AppModule（自动完成所有依赖注入）
let app_module = AppModuleManual::new(pool, config);

// 3. 在 Handler 中使用服务
let user_service = &state.module.user_service;
user_service.register(dto).await?;
```

## 📊 数据模型

### 用户 (User)
- 支持手机号/邮箱登录
- 可设置常驻城市（用于个性化推荐）

### 模板 (Template)
- 包含标题、描述、地理标签
- 步骤清单以 JSON 格式存储
- 支持父子继承关系（为未来扩展）

### 用户清单 (UserChecklist)
- Fork 自模板
- 记录每个步骤的完成状态
- 自动计算完成进度

## 🗺️ Roadmap

### V0.0.1 (当前版本) ✅
- [x] 基础架构搭建
- [x] 用户系统
- [x] 模板浏览和搜索
- [x] Fork 和进度追踪

### V0.1 (计划中)
- [ ] 用户创建和编辑模板
- [ ] 版本控制和修改建议
- [ ] 社区化功能

### V0.2 (未来)
- [ ] 个性化清单修改
- [ ] 步骤依赖关系
- [ ] 基于 LBS 的智能推荐

### V1.0 (愿景)
- [ ] 游戏化成就系统
- [ ] 城市主理人社区
- [ ] 评论和问答功能

## 📝 开发指南

### 添加新功能

1. 在 `models` crate 中定义数据模型
2. 在 `db` crate 中实现 Repository
3. 在 `core` crate 中实现 Service
4. 在 `api` crate 中实现 Handler
5. 更新 DI 配置（如需要）

### 数据库迁移

**✨ 自动迁移**: 应用启动时会自动运行所有pending迁移，通常不需要手动操作。

手动操作（可选）：

```bash
# 创建新迁移文件
sqlx migrate add <migration_name>

# 手动运行迁移（可选，应用启动时会自动运行）
sqlx migrate run

# 回滚最后一个迁移（谨慎使用）
sqlx migrate revert
```

## 📄 License

MIT License

## 👥 贡献

欢迎提交 Issue 和 Pull Request！

## 📧 联系方式

如有问题，请创建 Issue 或联系项目维护者。

