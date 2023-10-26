use crate::entities::{
    model::Model as ModelModel, query::Model as QueryModel, session::Model as SessionModel,
    user::Model as UserModel,
};
use crate::{
    database::{database_context::DatabaseContext, entity_context::EntityContextTrait},
    entities::access::Entity as AccessEntity,
    entities::in_use::Entity as InUseEntity,
    entities::model::Entity as ModelEntity,
    entities::query::Entity as QueryEntity,
    entities::session::Entity as SessionEntity,
    entities::user::Entity as UserEntity,
};
use sea_orm::sea_query::TableCreateStatement;
use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Schema};

pub async fn setup_db_with_entities(entities: Vec<AnyEntity>) -> Box<DatabaseContext> {
    let connection = Database::connect("sqlite::memory:").await.unwrap();
    let schema = Schema::new(DatabaseBackend::Sqlite);
    for entity in entities.iter() {
        entity.create_table_from(&connection, &schema).await;
    }
    Box::new(DatabaseContext {
        db_connection: connection,
    })
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
            AnyEntity::User => schema.create_table_from_entity(UserEntity),
            AnyEntity::Model => schema.create_table_from_entity(ModelEntity),
            AnyEntity::Access => schema.create_table_from_entity(AccessEntity),
            AnyEntity::Session => schema.create_table_from_entity(SessionEntity),
            AnyEntity::InUse => schema.create_table_from_entity(InUseEntity),
            AnyEntity::Query => schema.create_table_from_entity(QueryEntity),
        };
        connection
            .execute(connection.get_database_backend().build(&stmt))
            .await
            .unwrap();
    }
}
#[deprecated] // tihi :)))
pub fn create_users(amount: i32) -> Vec<UserModel> {
    let mut vector: Vec<UserModel> = vec![];
    for i in 0..amount {
        vector.push(UserModel {
            id: i + 1,
            email: format!("mail{}@mail.dk", &i),
            username: format!("username{}", &i),
            password: format!("qwerty{}", &i),
        })
    }
    vector
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
#[allow(dead_code)]
pub fn create_entities<M, F>(amount: i32, model: F) -> Vec<M>
where
    F: Fn(i32) -> M,
{
    let mut vector: Vec<M> = vec![];
    for i in 0..amount {
        vector.push(model(i));
    }
    vector
}
