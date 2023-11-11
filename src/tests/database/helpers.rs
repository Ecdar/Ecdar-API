#![cfg(test)]

use crate::database::database_context::{
    DatabaseContextTrait, PostgresDatabaseContext, SQLiteDatabaseContext,
};
use crate::entities::{access, in_use, model, query, session, user};
use dotenv::dotenv;
use std::env;
use uuid::Uuid;

pub async fn get_reset_database_context() -> Box<dyn DatabaseContextTrait> {
    dotenv().ok();

    let url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests.");

    let db_context: Box<dyn DatabaseContextTrait> = match url.split_at(url.find(":").unwrap()).0 {
        "sqlite" => Box::new(SQLiteDatabaseContext::new().await.unwrap()),
        "postgresql" => Box::new(PostgresDatabaseContext::new().await.unwrap()),
        _ => {
            panic!("Tests do not support the database protocol")
        }
    };

    db_context.reset().await.unwrap()
}

///
///
/// # Arguments
///
/// * `amount`:
/// * `model`:
///
/// returns: Vec<M, Global>
///
/// # Examples
///
/// ```
/// let vector: Vec<UserModel> = create_entities(3,|x| UserModel {
///     id: &x+i,
///     email: format!("mail{}@mail.dk",&x),
///     username: format!("username{}", &x),
///     password: format!("qwerty{}", &x),
/// );
/// ```

pub fn create_entities<M, F>(amount: i32, model_creator: F) -> Vec<M>
where
    F: Fn(i32) -> M,
{
    let mut vector: Vec<M> = vec![];
    for i in 0..amount {
        vector.push(model_creator(i));
    }
    vector
}

pub fn create_users(amount: i32) -> Vec<user::Model> {
    create_entities(amount, |i| user::Model {
        id: i + 1,
        email: format!("mail{}@mail.dk", &i),
        username: format!("username{}", &i),
        password: format!("qwerty{}", &i),
    })
}

pub fn create_models(amount: i32, user_id: i32) -> Vec<model::Model> {
    create_entities(amount, |i| model::Model {
        id: i + 1,
        name: "name".to_string(),
        components_info: "{}".to_owned().parse().unwrap(),
        owner_id: user_id,
    })
}

pub fn create_accesses(amount: i32, user_id: i32, model_id: i32) -> Vec<access::Model> {
    create_entities(amount, |i| access::Model {
        id: i + 1,
        role: "Reader".into(),
        model_id: model_id + i,
        user_id: user_id + i,
    })
}

pub fn create_sessions(amount: i32, user_id: i32) -> Vec<session::Model> {
    create_entities(amount, |i| session::Model {
        id: i + 1,
        token: Uuid::new_v4(),
        user_id,
        created_at: Default::default(),
    })
}

pub fn create_in_uses(amount: i32, model_id: i32, session_id: i32) -> Vec<in_use::Model> {
    create_entities(amount, |i| in_use::Model {
        model_id: model_id + i,
        session_id,
        latest_activity: Default::default(),
    })
}

pub fn create_queries(amount: i32, model_id: i32) -> Vec<query::Model> {
    create_entities(amount, |i| query::Model {
        id: i + 1,
        string: "".to_string(),
        result: None,
        outdated: true,
        model_id,
    })
}

#[macro_export]
macro_rules! to_active_models {
    ($vec:expr) => {{
        let mut models = Vec::new();
        for model in $vec {
            models.push(model.into_active_model());
        }
        models
    }};
}
