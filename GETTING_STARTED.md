# 🚀 快速开始指南

## 环境准备

### 必需软件

1. **Rust** (1.75+)
```bash
# 安装rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows用户访问
https://rustup.rs/
```

2. **PostgreSQL** (14+)
```bash
# 方式1: 使用Docker (推荐)
docker-compose up -d

# 方式2: 本地安装
# macOS
brew install postgresql@14

# Ubuntu
sudo apt install postgresql-14

# Windows
# 下载安装包: https://www.postgresql.org/download/windows/
```

3. **SQLx CLI**
```bash
cargo install sqlx-cli --no-default-features --features postgres
```

## 快速启动

### 1. 克隆项目
```bash
git clone <your-repo-url>
cd rookie-guide
```

### 2. 配置环境变量
```bash
# 复制环境变量模板
cp .env.example .env

# 编辑.env文件，设置数据库连接
# DATABASE_HOST=localhost
# DATABASE_PORT=5432
# DATABASE_USER=postgres
# DATABASE_PASSWORD=password
# DATABASE_NAME=rookie_guide
# JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
```

### 3. 启动 PostgreSQL 服务

**使用Docker（推荐）**
```bash
docker-compose up -d
```

**或使用本地PostgreSQL**
```bash
# 确保PostgreSQL服务已启动
# Windows: net start postgresql-x64-14
# macOS/Linux: sudo systemctl start postgresql
```

### 4. 启动服务（自动完成数据库初始化）

```bash
# 直接运行
cargo run -p api

# 或使用Makefile
make dev
```

**✨ 自动完成**：
- 📦 自动创建数据库（如果不存在）
- 🔄 自动运行数据库迁移
- 🚀 启动API服务器

服务将在 `http://127.0.0.1:8080` 启动

## ✅ 验证安装

### 健康检查
```bash
curl http://127.0.0.1:8080/health
```

期望输出:
```json
{
  "status": "ok",
  "service": "rookie-guide-api",
  "version": "0.0.1"
}
```

## 📚 API示例

### 1. 注册用户
```bash
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "13800138000",
    "password": "password123",
    "nickname": "张三"
  }'
```

响应:
```json
{
  "user": {
    "id": "uuid-here",
    "nickname": "张三",
    "avatar_url": null,
    "home_city": null
  },
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc..."
}
```

### 2. 登录
```bash
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "phone": "13800138000",
    "password": "password123"
  }'
```

### 3. 创建模板（需要认证）
```bash
TOKEN="your-jwt-token"

curl -X POST http://127.0.0.1:8080/api/templates \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
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
      },
      {
        "title": "实地看房",
        "description": "检查房屋设施和周边环境",
        "order": 2
      },
      {
        "title": "签订合同",
        "description": "仔细阅读合同条款",
        "order": 3
      }
    ]
  }'
```

### 4. 列出所有模板
```bash
curl http://127.0.0.1:8080/api/templates?page=1&page_size=20
```

### 5. Fork模板到个人清单
```bash
TEMPLATE_ID="template-uuid"

curl -X POST http://127.0.0.1:8080/api/checklists \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "template_id": "'$TEMPLATE_ID'"
  }'
```

### 6. 更新步骤状态
```bash
CHECKLIST_ID="checklist-uuid"

curl -X PUT http://127.0.0.1:8080/api/checklists/$CHECKLIST_ID/steps \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "step_index": 0,
    "completed": true
  }'
```

### 7. 查看个人所有清单
```bash
curl http://127.0.0.1:8080/api/checklists \
  -H "Authorization: Bearer $TOKEN"
```

## 🔧 开发命令

```bash
# 检查代码
cargo check --workspace

# 运行测试
cargo test --workspace

# 构建release版本
cargo build --release --workspace

# 格式化代码
cargo fmt --all

# 运行linter
cargo clippy --all-targets --all-features

# 查看依赖树
cargo tree
```

## 📊 数据库管理

```bash
# 创建新迁移
sqlx migrate add <migration_name>

# 运行迁移
sqlx migrate run

# 回滚最后一个迁移
sqlx migrate revert

# 查看迁移状态
sqlx migrate info
```

## 🐛 常见问题

### 1. 数据库连接失败
```
Error: Failed to connect to database
```

**解决方案:**
- 检查PostgreSQL是否启动: `docker ps` 或 `pg_isctl status`
- 验证DATABASE_HOST、DATABASE_PORT等配置
- 检查防火墙设置

### 2. 编译错误: `core` not found
```
error: could not find `core`
```

**解决方案:**
- 这通常是缓存问题
- 运行 `cargo clean`
- 重新构建 `cargo build`

### 3. 迁移失败
```
Error: Migration failed
```

**解决方案:**
- 检查数据库是否存在
- 删除旧的迁移记录表: `DROP TABLE _sqlx_migrations;`
- 重新运行迁移

### 4. JWT认证失败
```
401 Unauthorized: Invalid token
```

**解决方案:**
- 检查JWT_SECRET是否一致
- 验证token是否过期
- 确保Authorization header格式: `Bearer <token>`

## 📖 下一步

- 阅读 [项目架构文档](PROJECT_STRUCTURE.md)
- 查看 [API文档](README.md#api-文档)
- 了解 [依赖注入模式](PROJECT_STRUCTURE.md#依赖注入-di-模式)

## 💡 提示

1. **开发时使用 `cargo watch`**
```bash
cargo install cargo-watch
cargo watch -x 'run -p api'
```

2. **使用 `just` 简化命令**
```bash
cargo install just
# 创建justfile with常用命令
```

3. **配置IDE**
- VS Code: 安装 `rust-analyzer` 插件
- IntelliJ: 安装 Rust 插件

4. **数据库GUI工具**
- DBeaver: https://dbeaver.io/
- pgAdmin: https://www.pgadmin.org/
- TablePlus: https://tableplus.com/

---

**需要帮助?** 创建一个Issue或查看项目文档

