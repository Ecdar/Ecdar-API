#![cfg(test)]
use crate::database::database_context::DatabaseContext;
use crate::entities::sea_orm_active_enums::Role;
use crate::entities::{access, in_use, model, query, session, user};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};
use std::env;
use uuid::Uuid;

pub fn get_database_backend() -> DatabaseBackend {
    let url = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests.");
    if url.starts_with("sqlite") {
        DatabaseBackend::Sqlite
    } else if url.starts_with("postgresql") {
        DatabaseBackend::Postgres
    } else {
        panic!("Unsupported database connection string.")
    }
}

pub async fn setup_db_with_entities(entities: Vec<AnyEntity>) -> Box<DatabaseContext> {
    let database_backend = get_database_backend();

    let schema = Schema::new(database_backend);

    match database_backend {
        DatabaseBackend::Postgres => {
            let conn_string = env::var("TEST_DATABASE_URL").unwrap();

            let db_connection = Database::connect(conn_string).await.unwrap();

            Migrator::fresh(&db_connection).await.unwrap();

            Box::new(DatabaseContext { db_connection })
        }
        DatabaseBackend::Sqlite => {
            let connection = Database::connect(env::var("TEST_DATABASE_URL").unwrap())
                .await
                .unwrap();

            for entity in entities.iter() {
                entity.create_table_from(&connection, &schema).await;
            }
            Box::new(DatabaseContext {
                db_connection: connection,
            })
        }
        _ => panic!("Database backend not supported"),
    }
}

pub enum AnyEntity {
    User,
    Model,
    Access,
    Session,
    InUse,
    Query,
}

impl AnyEntity {
    async fn create_table_from(&self, connection: &DatabaseConnection, schema: &Schema) {
        let stmt = match self {
            AnyEntity::User => schema.create_table_from_entity(user::Entity),
            AnyEntity::Model => schema.create_table_from_entity(model::Entity),
            AnyEntity::Access => schema.create_table_from_entity(access::Entity),
            AnyEntity::Session => schema.create_table_from_entity(session::Entity),
            AnyEntity::InUse => schema.create_table_from_entity(in_use::Entity),
            AnyEntity::Query => schema.create_table_from_entity(query::Entity),
        };
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await
            .unwrap();
    }
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
        role: Role::Commenter,
        model_id,
        user_id,
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
