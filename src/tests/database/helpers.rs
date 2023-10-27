use crate::entities::sea_orm_active_enums::Role;
use crate::entities::{
    access::Model as AccessModel, in_use::Model as InUseModel, model::Model as ModelModel,
    query::Model as QueryModel, session::Model as SessionModel, user::Model as UserModel,
};
use crate::{
    database::database_context::DatabaseContext, entities::access::Entity as AccessEntity,
    entities::in_use::Entity as InUseEntity, entities::model::Entity as ModelEntity,
    entities::query::Entity as QueryEntity, entities::session::Entity as SessionEntity,
    entities::user::Entity as UserEntity,
};
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

pub fn create_users(amount: i32) -> Vec<UserModel> {
    create_entities(amount, |i| UserModel {
        id: i + 1,
        email: format!("mail{}@mail.dk", &i),
        username: format!("username{}", &i),
        password: format!("qwerty{}", &i),
    })
}

pub fn create_models(amount: i32, user_id: i32) -> Vec<ModelModel> {
    create_entities(amount, |i| ModelModel {
        id: i + 1,
        name: "name".to_string(),
        components_info: "{}".to_owned().parse().unwrap(),
        owner_id: user_id,
    })
}

pub fn create_accesses(amount: i32, user_id: i32, model_id: i32) -> Vec<AccessModel> {
    create_entities(amount, |i| AccessModel {
        id: i + 1,
        role: Role::Commenter,
        model_id,
        user_id,
    })
}

pub fn create_sessions(amount: i32, user_id: i32) -> Vec<SessionModel> {
    create_entities(amount, |i| SessionModel {
        id: i + 1,
        token: Default::default(),
        user_id,
        created_at: Default::default(),
    })
}

pub fn create_in_use(amount: i32, model_id: i32, session_id: i32) -> Vec<InUseModel> {
    create_entities(amount, |_| InUseModel {
        model_id,
        session_id,
        latest_activity: Default::default(),
    })
}

pub fn create_query(amount: i32, model_id: i32) -> Vec<QueryModel> {
    create_entities(amount, |i| QueryModel {
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
