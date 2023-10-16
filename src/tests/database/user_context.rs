use crate::database::user_context;
use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::User;
use crate::entities::user::{ActiveModel, Model};

#[cfg(test)]
mod database_tests {
    use sea_orm::{
        entity::prelude::*, entity::*, tests_cfg::*,
        DatabaseBackend, MockDatabase, Transaction,
    };
    use crate::entities::user;

    #[tokio::test]
    async fn create_test() -> Result<(),DbErr> {
        let db = MockDatabase::new(DatabaseBackend::Postgres).append_query_results([
            vec![user::Model{
                id: 1,
                email: "anders21@student.aau.dk".to_owned(),
                username: "andemad".to_owned(),
                password: "rask".to_owned(),
            }],
            vec![user::Model{
                id: 1,
                email: "anders21@student.aau.dk".to_owned(),
                username: "andemad".to_owned(),
                password: "rask".to_owned(),},
                user::Model{
                    id: 2,
                    email: "andeand@and.and".to_owned(),
                    username: "OgsåAndersRask".to_owned(),
                    password: "rask".to_owned(),
                }
            ]
        ]);
        todo!()
    }


}