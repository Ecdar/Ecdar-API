use sea_orm_migration::prelude::*;

use super::m20231012_094422_create_session_table::Session;
use super::m20231012_094228_create_model_table::Model;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InUse::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InUse::ModelId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InUse::SessionId).integer().not_null())
                    .col(
                        ColumnDef::new(InUse::LatestActivity)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InUse::Table, InUse::ModelId)
                            .to(Model::Table, Model::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InUse::Table, InUse::SessionId)
                            .to(Session::Table, Session::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InUse::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum InUse {
    Table,
    ModelId,
    SessionId,
    LatestActivity,
}
