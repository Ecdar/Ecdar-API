use chrono::Local;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, NotSet, QueryFilter};
use std::sync::Arc;

use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::session;

pub struct SessionContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
pub trait SessionContextTrait: EntityContextTrait<session::Model> {
    async fn get_by_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<Option<session::Model>, DbErr>;
}

#[async_trait]
impl SessionContextTrait for SessionContext {
    async fn get_by_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<Option<session::Model>, DbErr> {
        session::Entity::find()
            .filter(session::Column::RefreshToken.eq(refresh_token))
            .one(&self.db_context.get_connection())
            .await
    }
}

impl SessionContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> Self {
        SessionContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<session::Model> for SessionContext {
    /// Creates a new session in the database based on the provided model.
    /// # Example
    /// ```rust
    /// use crate::entities::session::{Entity, Model};
    ///
    /// let new_session = Model {
    ///         id: 1,
    ///         token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
    ///         created_at: Local::now().naive_utc(),
    ///         user_id,
    ///     };
    /// let created_session = session_context.create(model).await.unwrap();
    /// ```
    async fn create(&self, entity: session::Model) -> Result<session::Model, DbErr> {
        let session = session::ActiveModel {
            id: Default::default(),
            refresh_token: Set(entity.refresh_token),
            access_token: Set(entity.access_token),
            user_id: Set(entity.user_id),
            updated_at: NotSet,
        };

        let session = session.insert(&self.db_context.get_connection()).await;
        session
    }

    /// Returns a session by searching for its id.
    /// # Example
    /// ```rust
    /// let session: Result<Option<Model>, DbErr> = session_context.get_by_id(id).await;
    /// ```
    async fn get_by_id(&self, id: i32) -> Result<Option<session::Model>, DbErr> {
        session::Entity::find_by_id(id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all models in a vector.
    /// # Example
    /// ```rust
    /// let session: Result<Vec<Model>, DbErr> = session_context.get_all().await;
    /// ```
    async fn get_all(&self) -> Result<Vec<session::Model>, DbErr> {
        session::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates a model in the database based on the provided model.
    /// # **Example**
    /// ## ***Model in database***
    /// ### Model table ###
    /// | id | token                                | created_at                | user_id |
    /// |----|--------------------------------------|---------------------------|---------|
    /// | 1  | 25b14ea1-7b78-4714-b3d0-35d9f56e6cb3 | 2023-09-22T12:42:13+02:00 | 2       |
    /// ## ***Rust code***
    /// ```rust
    /// use crate::entities::session::{Entity, Model};
    ///
    /// let new_session = Model {
    ///         id: 1,
    ///         token: Uuid::parse_str("4473240f-2acb-422f-bd1a-5214554ed0e0").unwrap(),
    ///         created_at: Local::now().naive_utc(),
    ///         user_id: 2,
    ///     };
    /// let created_session = session_context.create(model).await.unwrap();
    /// ```
    /// ## ***Result***
    /// ### Model table ###
    /// | id | token                                | created_at                | user_id |
    /// |----|--------------------------------------|---------------------------|---------|
    /// | 1  | 4473240f-2acb-422f-bd1a-5214554ed0e0 | 2023-10-24T13:49:16+02:00 | 2       |
    async fn update(&self, entity: session::Model) -> Result<session::Model, DbErr> {
        session::ActiveModel {
            id: Unchanged(entity.id),
            refresh_token: Set(entity.refresh_token),
            access_token: Set(entity.access_token),
            user_id: Unchanged(entity.user_id),
            updated_at: Set(Local::now().naive_local()),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Deletes a model in the database with a specific id.
    /// # **Example**
    /// ## ***Model in database***
    /// ### Model table ###
    /// | id | token                                | created_at                | user_id |
    /// |----|--------------------------------------|---------------------------|---------|
    /// | 1  | 25b14ea1-7b78-4714-b3d0-35d9f56e6cb3 | 2023-10-24T14:03:37+02:00 | 2       |
    /// ## ***Rust code***
    /// ```rust
    /// let deleted_session = session_context.delete(1).await.unwrap();
    /// ```
    /// ## ***Result***
    /// ### Model table ###
    /// | id | token | created_at | user_id |
    /// |----|-------|------------|---------|
    /// |    |       |            |         |
    async fn delete(&self, id: i32) -> Result<session::Model, DbErr> {
        let session = self.get_by_id(id).await?;
        match session {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(session) => {
                session::Entity::delete_by_id(id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(session)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/session_context.rs"]
mod tests;
