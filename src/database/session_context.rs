use std::fmt::Debug;

use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, QueryFilter, ColumnTrait};

use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::Session as SessionEntity;
use crate::entities::session::{ActiveModel, Model as Session};
use crate::entities::session::Column as SessionColumn;

#[derive(Debug)]
pub struct SessionContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

#[async_trait]
pub trait SessionContextTrait: EntityContextTrait<Session> {
    async fn get_by_refresh_token(&self, refresh_token: String) -> Result<Option<Session>, DbErr>;
}

#[async_trait]
impl SessionContextTrait for SessionContext {
    /// Returns a session by searching for its refresh_token.
    /// # Example
    /// ```rust
    /// let session: Result<Option<Model>, DbErr> = session_context.get_by_refresh_token(refresh_token).await;
    /// ```
    async fn get_by_refresh_token(&self, refresh_token: String) -> Result<Option<Session>, DbErr> {
        SessionEntity::find()
            .filter(SessionColumn::RefreshToken.eq(refresh_token))
            .one(&self.db_context.get_connection())
            .await
    }
}

#[async_trait]
impl EntityContextTrait<Session> for SessionContext {
    /// Creates a new `SessionContext` for interacting with the database.
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> Self {
        SessionContext { db_context }
    }
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
    async fn create(&self, entity: Session) -> Result<Session, DbErr> {
        let session = ActiveModel {
            id: Default::default(),
            refresh_token: Set(entity.refresh_token),
            access_token: Set(entity.access_token),
            updated_at: Set(entity.updated_at),
            user_id: Set(entity.user_id),
        };

        session.insert(&self.db_context.get_connection()).await
    }

    /// Returns a session by searching for its id.
    /// # Example
    /// ```rust
    /// let session: Result<Option<Model>, DbErr> = session_context.get_by_id(id).await;
    /// ```
    async fn get_by_id(&self, id: i32) -> Result<Option<Session>, DbErr> {
        SessionEntity::find_by_id(id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all models in a vector.
    /// # Example
    /// ```rust
    /// let session: Result<Vec<Model>, DbErr> = session_context.get_all().await;
    /// ```
    async fn get_all(&self) -> Result<Vec<Session>, DbErr> {
        SessionEntity::find()
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
    async fn update(&self, entity: Session) -> Result<Session, DbErr> {
        let res = &self.get_by_id(entity.id).await?;
        let updated_session: Result<Session, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(format!(
                "Could not find entity {:?}",
                entity
            ))),
            Some(session) => {
                ActiveModel {
                    id: Unchanged(session.id),
                    refresh_token: Set(entity.refresh_token),
                    access_token: Set(entity.access_token),
                    updated_at: Set(entity.updated_at),
                    user_id: Unchanged(session.user_id), //TODO Should it be allowed to change the user_id of a session?
                }
                .update(&self.db_context.get_connection())
                .await
            }
        };
        return updated_session;
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
    async fn delete(&self, id: i32) -> Result<Session, DbErr> {
        let session = self.get_by_id(id).await?;
        match session {
            None => Err(DbErr::Exec(sea_orm::RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(session) => {
                SessionEntity::delete_by_id(id)
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
