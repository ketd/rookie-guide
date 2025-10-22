use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 user_checklists 表
        manager
            .create_table(
                Table::create()
                    .table(UserChecklists::Table)
                    .if_not_exists()
                    .col(uuid(UserChecklists::Id).primary_key())
                    .col(uuid(UserChecklists::UserId))
                    .col(uuid(UserChecklists::SourceTemplateId))
                    .col(string_len(UserChecklists::Title, 255))
                    .col(json_binary(UserChecklists::ProgressStatus)) // JSONB
                    .col(timestamp_with_time_zone(UserChecklists::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(UserChecklists::UpdatedAt).default(Expr::current_timestamp()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_checklists_user_id")
                            .from(UserChecklists::Table, UserChecklists::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_checklists_source_template_id")
                            .from(UserChecklists::Table, UserChecklists::SourceTemplateId)
                            .to(Templates::Table, Templates::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_user_checklists_user_id")
                    .table(UserChecklists::Table)
                    .col(UserChecklists::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_checklists_source_template")
                    .table(UserChecklists::Table)
                    .col(UserChecklists::SourceTemplateId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_user_checklists_created_at")
                    .table(UserChecklists::Table)
                    .col(UserChecklists::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserChecklists::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserChecklists {
    Table,
    Id,
    UserId,
    SourceTemplateId,
    Title,
    ProgressStatus,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Templates {
    Table,
    Id,
}

