use sea_orm_migration::prelude::*;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_type(
                Type::create()
                    .as_enum(Role::Table)
                    .values(Role::iter().skip(1))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Role::Table).to_owned())
            .await
    }
}

#[derive(Iden, EnumIter)]
pub enum Role {
    Table,
    #[iden = "Reader"]
    Reader,
    #[iden = "Commenter"]
    Commenter,
    #[iden = "Editor"]
    Editor,
}