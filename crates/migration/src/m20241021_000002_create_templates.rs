use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 templates 表
        manager
            .create_table(
                Table::create()
                    .table(Templates::Table)
                    .if_not_exists()
                    .col(uuid(Templates::Id).primary_key())
                    .col(string_len(Templates::Title, 255))
                    .col(text(Templates::Description))
                    .col(string_len(Templates::LocationTag, 50))
                    .col(json_binary(Templates::Steps)) // JSONB
                    .col(uuid_null(Templates::ParentId))
                    .col(uuid(Templates::CreatedBy))
                    .col(boolean(Templates::IsOfficial).default(false))
                    .col(timestamp_with_time_zone(Templates::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(Templates::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_templates_parent_id")
                            .from(Templates::Table, Templates::ParentId)
                            .to(Templates::Table, Templates::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_templates_created_by")
                            .from(Templates::Table, Templates::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_templates_location_tag")
                    .table(Templates::Table)
                    .col(Templates::LocationTag)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_templates_created_by")
                    .table(Templates::Table)
                    .col(Templates::CreatedBy)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_templates_is_official")
                    .table(Templates::Table)
                    .col(Templates::IsOfficial)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_templates_created_at")
                    .table(Templates::Table)
                    .col(Templates::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_templates_parent_id")
                    .table(Templates::Table)
                    .col(Templates::ParentId)
                    .to_owned(),
            )
            .await?;

        // 创建全文搜索索引（PostgreSQL specific）
        // 注意：SeaORM 不直接支持 GIN 索引，需要使用原生 SQL
        let sql = r#"
            CREATE INDEX idx_templates_search ON templates 
            USING gin(to_tsvector('simple', title || ' ' || description))
        "#;
        
        manager.get_connection().execute_unprepared(sql).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Templates::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Templates {
    Table,
    Id,
    Title,
    Description,
    LocationTag,
    Steps,
    ParentId,
    CreatedBy,
    IsOfficial,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

