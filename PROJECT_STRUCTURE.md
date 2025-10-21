# 阅历进度条 - 项目架构文档

## 📁 项目结构

```
rookie-guide/
├── Cargo.toml              # Workspace配置
├── .env.example            # 环境变量模板
├── docker-compose.yml      # PostgreSQL容器配置
├── Makefile               # 便捷命令
├── README.md              # 项目说明
├── crates/
│   ├── api/               # 🌐 Web API服务层
│   │   ├── src/
│   │   │   ├── main.rs           # 应用入口
│   │   │   ├── state.rs          # 应用状态（包含DI容器）
│   │   │   ├── routes.rs         # 路由配置
│   │   │   ├── handlers/         # HTTP处理器
│   │   │   │   ├── health.rs     # 健康检查
│   │   │   │   ├── auth.rs       # 认证相关
│   │   │   │   ├── user.rs       # 用户相关
│   │   │   │   ├── template.rs   # 模板相关
│   │   │   │   └── checklist.rs  # 清单相关
│   │   │   └── middleware/       # 中间件
│   │   │       └── auth.rs       # JWT认证中间件
│   │   └── Cargo.toml
│   │
│   ├── service_layer/     # 🎯 核心业务逻辑层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── di.rs             # 依赖注入容器
│   │   │   └── services/         # 业务服务
│   │   │       ├── template_service.rs
│   │   │       ├── user_service.rs
│   │   │       └── checklist_service.rs
│   │   └── Cargo.toml
│   │
│   ├── db/                # 💾 数据库访问层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pool.rs           # 数据库连接池
│   │   │   └── repositories/     # Repository实现
│   │   │       ├── template_repository.rs
│   │   │       ├── user_repository.rs
│   │   │       └── user_checklist_repository.rs
│   │   └── Cargo.toml
│   │
│   ├── models/            # 📦 数据模型层
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── template.rs       # 模板模型 + DTO
│   │   │   ├── user.rs           # 用户模型 + DTO
│   │   │   └── user_checklist.rs # 清单模型 + DTO
│   │   └── Cargo.toml
│   │
│   ├── auth/              # 🔐 认证模块
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── jwt.rs            # JWT服务
│   │   │   └── password.rs       # 密码服务
│   │   └── Cargo.toml
│   │
│   └── common/            # 🛠️ 公共工具
│       ├── src/
│       │   ├── lib.rs
│       │   ├── config.rs         # 配置管理
│       │   └── error.rs          # 错误类型
│       └── Cargo.toml
│
└── migrations/            # 📊 数据库迁移
    ├── 20241021_create_users.sql
    ├── 20241021_create_templates.sql
    └── 20241021_create_user_checklists.sql
```

## 🏗️ 架构设计

### 分层架构

```
┌─────────────────────────────────────┐
│      API Layer (handlers)          │  ← HTTP请求处理
├─────────────────────────────────────┤
│   Service Layer (business logic)   │  ← 业务逻辑
├─────────────────────────────────────┤
│  Repository Layer (data access)    │  ← 数据访问
├─────────────────────────────────────┤
│        Database (PostgreSQL)        │  ← 数据存储
└─────────────────────────────────────┘
```

### 依赖注入 (DI) 模式

项目采用**手动依赖注入**模式，使用trait对象和Arc实现：

```rust
// 1. 定义服务trait
pub trait UserService: Send + Sync {
    async fn register(&self, dto: RegisterDto) -> AppResult<AuthResponse>;
    // ...
}

// 2. 实现服务，依赖通过构造函数注入
pub struct UserServiceImpl {
    user_repo: Arc<dyn UserRepository>,
    jwt_service: Arc<dyn JwtService>,
    password_service: Arc<dyn PasswordService>,
}

// 3. 在AppModule中统一管理所有依赖
pub struct AppModule {
    pub template_service: Arc<dyn TemplateService>,
    pub user_service: Arc<dyn UserService>,
    pub checklist_service: Arc<dyn ChecklistService>,
}

impl AppModule {
    pub fn new(pool: PgPool, config: AppConfig) -> Self {
        // 按依赖层级创建实例
        // Layer 1: Repositories
        // Layer 2: Auth Services
        // Layer 3: Business Services
    }
}
```

**优势:**
- ✅ 类型安全
- ✅ 易于测试（可mock依赖）
- ✅ 清晰的依赖关系
- ✅ 编译时检查
- ✅ 无运行时开销

### 数据流向

```
HTTP Request
    ↓
Handler (提取参数)
    ↓
Service (业务逻辑 + 验证)
    ↓
Repository (数据库操作)
    ↓
Database
    ↓
Repository (返回数据)
    ↓
Service (业务处理)
    ↓
Handler (序列化响应)
    ↓
HTTP Response
```

## 🔑 核心概念

### 1. 阅历模板 (Template)
- 官方或社区创建的"母版攻略"
- 包含地理标签（通用/城市级）
- 步骤清单以JSON存储
- 支持父子继承（为未来扩展）

### 2. 用户清单 (UserChecklist)
- 从模板Fork而来
- 记录每个步骤的完成状态
- 自动计算完成进度
- 暂不支持修改（V0.0.1）

### 3. 依赖注入容器 (AppModule)
```rust
AppModule {
    template_service    // 依赖: TemplateRepository
    user_service        // 依赖: UserRepository, JwtService, PasswordService
    checklist_service   // 依赖: UserChecklistRepository, TemplateRepository
}
```

## 🎯 关键设计决策

### 为什么重命名 `core` → `service_layer`?
- Rust的`async_trait`宏内部使用`core::`前缀
- 与标准库的`core`模块冲突
- 重命名为`service_layer`更语义化且避免冲突

### 为什么使用手动DI而非框架?
- **简单直观**: 无需学习复杂的DI框架
- **类型安全**: 编译时检查所有依赖
- **灵活可控**: 完全控制依赖创建过程
- **运行时依赖**: PgPool等运行时创建的依赖更易处理

### 为什么选择trait + Arc?
- **多态**: 支持运行时替换实现（测试mock）
- **线程安全**: Arc允许跨线程共享
- **零成本抽象**: 性能接近直接调用

## 📦 Crate职责

| Crate | 职责 | 依赖 |
|-------|------|------|
| `api` | HTTP服务、路由、中间件 | service_layer, common, models, db, auth |
| `service_layer` | 核心业务逻辑 | db, auth, models, common |
| `db` | 数据库访问、Repository | models, common |
| `models` | 数据模型、DTO | - |
| `auth` | 认证、授权逻辑 | common |
| `common` | 工具、配置、错误 | - |

## 🚀 启动流程

1. **加载配置** (`AppConfig::from_env()`)
2. **创建数据库连接池** (`create_pool()`)
3. **运行数据库迁移** (`sqlx::migrate!()`)
4. **创建DI容器** (`AppModule::new()`)
5. **构建路由** (`routes::create_router()`)
6. **启动HTTP服务器** (`axum::serve()`)

## 🔒 认证流程

```
1. 用户登录 → POST /api/auth/login
2. 验证密码 → PasswordService::verify_password()
3. 生成JWT → JwtService::generate_token()
4. 返回token → { user, token }

受保护的请求:
1. 提取Authorization header
2. 验证JWT → JwtService::validate_token()
3. 提取CurrentUser → FromRequestParts实现
4. 注入到Handler
```

## 📊 数据库设计

### users 表
```sql
id UUID PRIMARY KEY
phone VARCHAR(20) UNIQUE
email VARCHAR(255) UNIQUE
password_hash VARCHAR(255) NOT NULL
nickname VARCHAR(100) NOT NULL
avatar_url TEXT
home_city VARCHAR(50)  -- 常驻城市（如"CN-BJ"）
created_at, updated_at
```

### templates 表
```sql
id UUID PRIMARY KEY
title VARCHAR(255) NOT NULL
description TEXT NOT NULL
location_tag VARCHAR(50) NOT NULL  -- 如"CN", "CN-BJ"
steps JSONB NOT NULL  -- TemplateStep数组
parent_id UUID  -- 父模板（继承）
created_by UUID REFERENCES users
is_official BOOLEAN DEFAULT FALSE
created_at, updated_at
```

### user_checklists 表
```sql
id UUID PRIMARY KEY
user_id UUID REFERENCES users
source_template_id UUID REFERENCES templates
title VARCHAR(255) NOT NULL  -- Fork时复制
progress_status JSONB NOT NULL  -- StepProgress数组
created_at, updated_at
```

## 🛠️ 开发工具

```bash
# 启动数据库
make docker-up

# 运行迁移
make migrate-up

# 启动开发服务器
make dev

# 构建release版本
make build

# 运行测试
make test

# 初始化项目（启动DB+迁移）
make init
```

## 📈 扩展方向

### V0.1 (社区化)
- 开放用户创建模板
- 版本控制
- 修改建议机制

### V0.2 (个性化)
- 自定义清单
- 步骤依赖关系
- LBS智能推荐

### V1.0 (生态)
- 游戏化系统
- 城市主理人
- 社交功能

---

**构建时间**: 2025-10-21  
**Rust版本**: 1.75+  
**数据库**: PostgreSQL 14+  

