# 数据库管理说明

## 📊 数据库技术选型

### SeaORM 全栈解决方案

本项目使用 **SeaORM 1.1** 作为完整的数据库解决方案：

| 组件 | 作用 | 优势 |
|------|------|------|
| **SeaORM** | 数据访问层（ORM） | ✅ 类型安全查询<br>✅ 实体关系映射<br>✅ 异步原生支持<br>✅ 编译时检查 |
| **SeaORM Migration** | 数据库迁移 | ✅ Rust代码定义<br>✅ 类型安全<br>✅ 自动执行<br>✅ 编译时验证 |
| **PostgreSQL 14+** | 数据库 | ✅ JSONB支持<br>✅ UUID类型<br>✅ 高性能<br>✅ 事务支持 |

### 为什么选择 SeaORM？

| 特性 | SeaORM | Diesel | SQLx Raw |
|------|--------|--------|----------|
| **异步支持** | ✅ 原生异步 | ⚠️ 需diesel-async | ✅ 原生异步 |
| **类型安全** | ✅ 编译时检查 | ✅ 编译时检查 | ⚠️ 宏检查 |
| **查询构建器** | ✅ 链式API | ✅ DSL | ❌ 手写SQL |
| **关系映射** | ✅ 声明式关系 | ✅ Joinable | ❌ 手动JOIN |
| **迁移管理** | ✅ Rust代码 | ✅ 自带CLI | ⚠️ 需额外工具 |
| **学习曲线** | ✅ 符合直觉 | ⚠️ 较陡峭 | ✅ 熟悉SQL即可 |

## 🔄 数据库先行（Database-First）

### 自动同步机制

**每次启动应用时，都会自动运行数据库迁移**，确保数据库结构与代码定义保持同步。

```rust
// 在 main.rs 中
use migration::{Migrator, MigratorTrait};

Migrator::up(&db, None).await?;
```

### 工作流程

```
应用启动
    ↓
连接数据库
    ↓
自动运行 SeaORM Migrations  ← 【数据库先行】
    ↓
初始化服务
    ↓
启动HTTP服务器
```

### 迁移状态追踪

SeaORM 会在数据库中创建 `seaql_migrations` 表来追踪已执行的迁移：

```sql
SELECT * FROM seaql_migrations;
```

| version | description | success | checksum | execution_time |
|---------|-------------|---------|----------|----------------|
| 20241021_create_users | ... | true | ... | 123ms |
| 20241021_create_templates | ... | true | ... | 89ms |

## 📝 迁移管理

### 创建新迁移

在 `crates/migration/src/` 目录下创建新的迁移文件：

```bash
# 文件命名规范: mYYYYMMDD_HHMMSS_description.rs
# 例如:
crates/migration/src/m20241022_120000_add_user_tags.rs
```

### 迁移文件示例

```rust
// crates/migration/src/m20241022_120000_add_user_tags.rs
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建用户标签表
        manager
            .create_table(
                Table::create()
                    .table(UserTags::Table)
                    .if_not_exists()
                    .col(uuid(UserTags::Id).primary_key())
                    .col(uuid(UserTags::UserId))
                    .col(string_len(UserTags::Tag, 50))
                    .col(timestamp_with_time_zone(UserTags::CreatedAt)
                        .default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserTags::Table, UserTags::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // 创建唯一索引（防止重复标签）
        manager
            .create_index(
                Index::create()
                    .name("idx_unique_user_tag")
                    .table(UserTags::Table)
                    .col(UserTags::UserId)
                    .col(UserTags::Tag)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserTags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserTags {
    Table,
    Id,
    UserId,
    Tag,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
```

### 注册迁移

在 `crates/migration/src/lib.rs` 中注册新迁移：

```rust
mod m20241021_000001_create_users;
mod m20241021_000002_create_templates;
mod m20241021_000003_create_user_checklists;
mod m20241022_120000_add_user_tags;  // 新增

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241021_000001_create_users::Migration),
            Box::new(m20241021_000002_create_templates::Migration),
            Box::new(m20241021_000003_create_user_checklists::Migration),
            Box::new(m20241022_120000_add_user_tags::Migration),  // 新增
        ]
    }
}
```

### 运行迁移

```bash
# 自动运行（应用启动时）
cargo run -p api

# 迁移会在应用启动时自动执行，无需手动操作！
```

## 🏗️ SeaORM 实体模型

### 实体定义示例

```rust
// crates/models/src/user.rs
use sea_orm::entity::prelude::*;

/// 用户实体模型
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    
    #[sea_orm(unique)]
    pub phone: String,
    
    pub nickname: String,
    pub password_hash: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

/// 定义实体关系
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::template::Entity")]
    Templates,
    
    #[sea_orm(has_many = "super::user_checklist::Entity")]
    Checklists,
}

impl ActiveModelBehavior for ActiveModel {}

// 类型别名（向后兼容）
pub type User = Model;
```

### Repository 层使用示例

```rust
// crates/db/src/repositories/user_repository.rs
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait};
use models::{User, UserEntity, UserColumn};

pub struct UserRepositoryImpl {
    db: DatabaseConnection,
}

impl UserRepository for UserRepositoryImpl {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        // SeaORM 查询：类型安全，编译时检查
        let user = UserEntity::find()
            .filter(UserColumn::Phone.eq(phone))
            .one(&self.db)
            .await?;
        
        Ok(user)
    }
    
    async fn create(&self, dto: RegisterDto, password_hash: String) -> AppResult<User> {
        use models::user::ActiveModel;
        
        // 使用 ActiveModel 创建新记录
        let active_model = ActiveModel {
            id: Set(Uuid::new_v4()),
            phone: Set(dto.phone),
            nickname: Set(dto.nickname),
            password_hash: Set(password_hash),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        };
        
        let user = active_model.insert(&self.db).await?;
        Ok(user)
    }
}
```

### JSONB 字段处理

```rust
// 实体定义
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "templates")]
pub struct Model {
    pub id: Uuid,
    pub title: String,
    
    // JSONB 字段：存储为 sea_orm::prelude::Json
    #[sea_orm(column_type = "Json")]
    pub steps: Json,
}

// 辅助方法：序列化/反序列化
impl Model {
    /// 获取步骤列表（反序列化）
    pub fn get_steps(&self) -> Result<Vec<TemplateStep>, serde_json::Error> {
        serde_json::from_value(self.steps.clone())
    }
    
    /// 设置步骤列表（序列化）
    pub fn set_steps(&mut self, steps: Vec<TemplateStep>) -> Result<(), serde_json::Error> {
        self.steps = serde_json::to_value(steps)?;
        Ok(())
    }
}
```

## 🛠️ 开发工作流

### 添加新表

1. **创建迁移文件**
```bash
# 在 crates/migration/src/ 创建新文件
# 例如: m20241022_130000_create_comments.rs
```

2. **编写迁移代码**
```rust
-- migrations/20241022_create_comments.sql
CREATE TABLE comments (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

3. **定义Rust模型**
```rust
// crates/models/src/comment.rs
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

4. **创建Repository**
```rust
// crates/db/src/repositories/comment_repository.rs
pub trait CommentRepository: Send + Sync {
    async fn create(&self, comment: CreateCommentDto) -> AppResult<Comment>;
    async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Comment>>;
}
```

5. **启动应用（自动运行迁移）**
```bash
cargo run -p api
# 输出：
# 🔄 开始同步数据库结构...
# ✅ 数据库结构同步完成
```

### 修改现有表

**重要**: SQLx迁移是**只能向前**的，不支持自动回滚。

#### 添加列（安全）
```sql
-- migrations/20241022_add_user_bio.sql
ALTER TABLE users 
ADD COLUMN bio TEXT;

-- 添加默认值（避免NOT NULL约束问题）
ALTER TABLE users 
ADD COLUMN verified BOOLEAN DEFAULT FALSE;
```

#### 删除列（谨慎！）
```sql
-- migrations/20241022_remove_user_old_field.sql
ALTER TABLE users 
DROP COLUMN IF EXISTS old_field;
```

#### 重命名列（需要数据迁移）
```sql
-- migrations/20241022_rename_user_name.sql

-- 步骤1: 添加新列
ALTER TABLE users ADD COLUMN full_name VARCHAR(100);

-- 步骤2: 复制数据
UPDATE users SET full_name = name WHERE full_name IS NULL;

-- 步骤3: 删除旧列（可选，在后续迁移中）
-- ALTER TABLE users DROP COLUMN name;
```

## ⚠️ 注意事项

### 1. 迁移不可变性

**迁移文件一旦运行就不应修改！**

```bash
# ❌ 错误做法
# 修改已运行的迁移文件会导致checksum不匹配

# ✅ 正确做法
# 创建新的迁移来修复错误
sqlx migrate add fix_previous_migration
```

### 2. 生产环境迁移

```bash
# 生产环境部署前
# 1. 备份数据库
pg_dump rookie_guide > backup.sql

# 2. 在staging环境测试迁移
DATABASE_HOST=staging.db.example.com cargo run -p api

# 3. 确认无误后部署生产环境
```

### 3. 数据迁移

对于复杂的数据迁移：

```sql
-- migrations/20241022_complex_data_migration.sql

-- 开启事务
BEGIN;

-- 数据转换
UPDATE users 
SET location_tag = CONCAT('CN-', UPPER(city_code))
WHERE city_code IS NOT NULL;

-- 验证
DO $$
BEGIN
    IF (SELECT COUNT(*) FROM users WHERE city_code IS NOT NULL AND location_tag IS NULL) > 0 THEN
        RAISE EXCEPTION '数据迁移失败：存在未转换的数据';
    END IF;
END $$;

-- 提交事务
COMMIT;
```

### 4. 性能考虑

对大表的迁移：

```sql
-- 添加索引时使用CONCURRENTLY（不锁表）
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);

-- 大数据量更新使用批处理
UPDATE users 
SET updated_at = NOW()
WHERE id IN (
    SELECT id FROM users 
    WHERE updated_at IS NULL 
    LIMIT 1000
);
```

## 🔍 调试迁移问题

### 查看迁移历史

```sql
-- 查看所有已执行的迁移
SELECT * FROM _sqlx_migrations ORDER BY installed_on DESC;

-- 查看最后一次迁移
SELECT * FROM _sqlx_migrations 
ORDER BY installed_on DESC 
LIMIT 1;
```

### 重置迁移（开发环境）

```bash
# ⚠️ 警告：这会删除所有数据！

# 方式1: 删除数据库重建
dropdb rookie_guide
createdb rookie_guide
cargo run -p api

# 方式2: 清空迁移表
psql rookie_guide -c "TRUNCATE _sqlx_migrations CASCADE;"
sqlx migrate run
```

### 迁移失败处理

```bash
# 如果迁移失败，检查：

# 1. 查看错误日志
cargo run -p api 2>&1 | grep "迁移失败"

# 2. 验证SQL语法
psql rookie_guide -f migrations/20241022_problematic.sql

# 3. 手动回滚（如果在事务中）
# 迁移会自动回滚，无需手动操作

# 4. 修复后创建新迁移
sqlx migrate add fix_issue
```

## 📚 最佳实践

### ✅ 推荐做法

1. **每个迁移做一件事**: 便于回溯和理解
2. **使用事务**: 确保原子性
3. **添加注释**: 说明迁移目的
4. **先测试后部署**: staging → production
5. **备份数据**: 特别是生产环境
6. **使用IF EXISTS/IF NOT EXISTS**: 提高幂等性

### ❌ 避免的做法

1. **修改已运行的迁移**: 会导致checksum错误
2. **在迁移中删除数据**: 除非绝对必要
3. **没有测试的迁移**: 可能导致生产事故
4. **长时间锁表**: 影响服务可用性

## 🔗 相关资源

- [SQLx文档](https://github.com/launchbadge/sqlx)
- [PostgreSQL文档](https://www.postgresql.org/docs/)
- [数据库迁移最佳实践](https://www.postgresql.org/docs/current/ddl-alter.html)

---

**记住**: 数据库迁移是**不可逆的**，请务必谨慎操作！

