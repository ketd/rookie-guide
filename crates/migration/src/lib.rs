pub use sea_orm_migration::prelude::*;

mod m20241021_000001_create_users;
mod m20241021_000002_create_templates;
mod m20241021_000003_create_user_checklists;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20241021_000001_create_users::Migration),
            Box::new(m20241021_000002_create_templates::Migration),
            Box::new(m20241021_000003_create_user_checklists::Migration),
        ]
    }
}

