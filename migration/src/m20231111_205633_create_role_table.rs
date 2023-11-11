use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RoleEnum::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RoleEnum::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RoleEnum::Name)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await;

        let insert = Query::insert()
            .into_table(RoleEnum::Table)
            .columns([RoleEnum::Name])
            .values_panic(["Editor".into()])
            .values_panic(["Reader".into()])
            .values_panic(["Comenter".into()])
            .to_owned();

        manager.exec_stmt(insert).await;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RoleEnum::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RoleEnum {
    Table,
    Id,
    Name,
}
