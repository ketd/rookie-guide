use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 users 表
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(uuid(Users::Id).primary_key())
                    .col(string_len_null(Users::Phone, 20).unique_key())
                    .col(string_len_null(Users::Email, 255).unique_key())
                    .col(string_len(Users::PasswordHash, 255))
                    .col(string_len(Users::Nickname, 100))
                    .col(text_null(Users::AvatarUrl))
                    .col(string_len_null(Users::HomeCity, 50))
                    .col(timestamp_with_time_zone(Users::CreatedAt).default(Expr::current_timestamp()))
                    .col(timestamp_with_time_zone(Users::UpdatedAt).default(Expr::current_timestamp()))
                    .check(
                        Expr::col(Users::Phone).is_not_null()
                            .or(Expr::col(Users::Email).is_not_null())
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_users_phone")
                    .table(Users::Table)
                    .col(Users::Phone)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_email")
                    .table(Users::Table)
                    .col(Users::Email)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_home_city")
                    .table(Users::Table)
                    .col(Users::HomeCity)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Phone,
    Email,
    PasswordHash,
    Nickname,
    AvatarUrl,
    HomeCity,
    CreatedAt,
    UpdatedAt,
}

