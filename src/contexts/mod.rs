pub mod context_collection;
pub mod context_impls;
pub mod context_traits;

#[cfg(test)]
mod helpers {
    use crate::contexts::context_impls::{PostgresDatabaseContext, SQLiteDatabaseContext};
    use crate::contexts::context_traits::DatabaseContextTrait;
    use crate::entities::{access, in_use, project, query, session, user};
    use dotenv::dotenv;
    use sea_orm::{ConnectionTrait, Database, DbBackend};
    use std::env;
    use std::sync::Arc;

    pub async fn get_reset_database_context() -> Arc<dyn DatabaseContextTrait> {
        dotenv().ok();

        let url =
            env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set to run tests.");
        let db = Database::connect(&url).await.unwrap();
        let db_context: Arc<dyn DatabaseContextTrait> = match db.get_database_backend() {
            DbBackend::Sqlite => Arc::new(SQLiteDatabaseContext::new(&url).await.unwrap()),
            DbBackend::Postgres => Arc::new(PostgresDatabaseContext::new(&url).await.unwrap()),
            _ => panic!("Database protocol not supported"),
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

    pub fn create_entities<M, F>(amount: i32, project_creator: F) -> Vec<M>
    where
        F: Fn(i32) -> M,
    {
        let mut vector: Vec<M> = vec![];
        for i in 0..amount {
            vector.push(project_creator(i));
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

    pub fn create_projects(amount: i32, user_id: i32) -> Vec<project::Model> {
        create_entities(amount, |i| project::Model {
            id: i + 1,
            name: format!("name {}", i),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: user_id,
        })
    }

    pub fn create_accesses(amount: i32, user_id: i32, project_id: i32) -> Vec<access::Model> {
        create_entities(amount, |i| access::Model {
            id: i + 1,
            role: "Reader".into(),
            project_id: project_id + i,
            user_id: user_id + i,
        })
    }

    pub fn create_sessions(amount: i32, user_id: i32) -> Vec<session::Model> {
        create_entities(amount, |i| session::Model {
            id: i + 1,
            refresh_token: "test_refresh_token".to_string() + format!("{}", i).as_str(),
            access_token: "test_access_token".to_string() + format!("{}", i).as_str(),
            user_id,
            updated_at: Default::default(),
        })
    }

    pub fn create_in_uses(amount: i32, project_id: i32, session_id: i32) -> Vec<in_use::Model> {
        create_entities(amount, |i| in_use::Model {
            project_id: project_id + i,
            session_id,
            latest_activity: Default::default(),
        })
    }

    pub fn create_queries(amount: i32, project_id: i32) -> Vec<query::Model> {
        create_entities(amount, |i| query::Model {
            id: i + 1,
            string: "".to_string(),
            result: None,
            outdated: true,
            project_id,
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
}
