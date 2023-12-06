use sea_orm_migration::prelude::*;

use super::m20231012_094213_create_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Project::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Project::Name).string().not_null())
                    .col(ColumnDef::new(Project::ComponentsInfo).json().not_null())
                    .col(ColumnDef::new(Project::OwnerId).integer().not_null())
                    .index(
                        Index::create()
                            .col(Project::OwnerId)
                            .col(Project::Name)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Project::Table, Project::OwnerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Project {
    Table,
    Id,
    Name,
    ComponentsInfo,
    OwnerId,
}
