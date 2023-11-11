pub use sea_orm_migration::prelude::*;

mod m20231012_094213_create_user_table;
mod m20231012_094228_create_model_table;
mod m20231012_094242_create_query_table;
mod m20231012_094303_create_in_use_table;
mod m20231012_094422_create_session_table;
mod m20231012_094533_create_access_table;
mod m20231111_205633_create_role_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231012_094213_create_user_table::Migration),
            Box::new(m20231012_094228_create_model_table::Migration),
            Box::new(m20231012_094242_create_query_table::Migration),
            Box::new(m20231012_094422_create_session_table::Migration),
            Box::new(m20231012_094303_create_in_use_table::Migration),
            Box::new(m20231111_205633_create_role_table::Migration),
            Box::new(m20231012_094533_create_access_table::Migration),
        ]
    }
}
