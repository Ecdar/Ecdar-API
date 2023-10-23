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
                    .table(Model::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Model::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Model::Name).string().not_null())
                    .col(ColumnDef::new(Model::ComponentsInfo).json().not_null())
                    .col(ColumnDef::new(Model::OwnerId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Model::Table, Model::OwnerId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Model::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Model {
    Table,
    Id,
    Name,
    ComponentsInfo,
    OwnerId,
}
