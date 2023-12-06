use sea_orm_migration::prelude::*;

use super::m20231012_094228_create_project_table::Project;
use super::m20231012_094422_create_session_table::Session;

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
                        ColumnDef::new(InUse::ProjectId)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InUse::SessionId).integer().not_null())
                    .col(
                        ColumnDef::new(InUse::LatestActivity)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InUse::Table, InUse::ProjectId)
                            .to(Project::Table, Project::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InUse::Table, InUse::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_delete(ForeignKeyAction::Cascade),
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
    ProjectId,
    SessionId,
    LatestActivity,
}
