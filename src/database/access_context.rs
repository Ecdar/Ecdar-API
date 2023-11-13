use std::fmt::Debug;

use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, RuntimeErr};

use crate::database::database_context::DatabaseContextTrait;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::access::{ActiveModel, Model as Access};
use crate::entities::prelude::Access as AccessEntity;

#[derive(Debug)]
pub struct AccessContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait AccessContextTrait: EntityContextTrait<Access> {}

impl AccessContextTrait for AccessContext {}

#[async_trait]
impl EntityContextTrait<Access> for AccessContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> AccessContext {
        AccessContext { db_context }
    }

    /// Used for creating an Access entity
    /// # Example
    /// ```
    /// let access = Access {
    ///     id: Default::default(),
    ///     role: Role::Editor,
    ///     user_id: 1,
    ///     model_id: 1
    /// };
    /// let context : AccessContext = AccessContext::new(...);
    /// context.create(model);
    /// ```
    async fn create(&self, entity: Access) -> Result<Access, DbErr> {
        let access = ActiveModel {
            id: Default::default(),
            role: Set(entity.role),
            model_id: Set(entity.model_id),
            user_id: Set(entity.user_id),
        };
        let access: Access = access.insert(&self.db_context.get_connection()).await?;
        Ok(access)
    }

    /// Returns a single access entity (uses primary key)
    /// # Example
    /// ```
    /// let context : AccessContext = AccessContext::new(...);
    /// let model : Model = context.get_by_id(1).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Access>, DbErr> {
        AccessEntity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all the access entities
    /// # Example
    /// ```
    /// let context : AccessContext = AccessContext::new(...);
    /// let model : vec<Model> = context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<Access>, DbErr> {
        AccessEntity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates and returns the given access entity
    /// # Example
    /// ```
    /// let context : AccessContext = AccessContext::new(...);
    /// let access = context.get_by_id(1).unwrap();
    /// let updated_access = Model {
    ///     id: access.id,
    ///     role: Role::Reader,
    ///     user_id: access.user_id,
    ///     model_id: access.model_id
    /// }
    /// ```
    /// # Note
    /// The access entity's ids will never be changed. If this behavior is wanted, delete the old access and create a new one.
    async fn update(&self, entity: Access) -> Result<Access, DbErr> {
        let res = &self.get_by_id(entity.id).await?;
        let updated_access: Result<Access, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(format!(
                "Could not find entity {:?}",
                entity
            ))),
            Some(access) => {
                ActiveModel {
                    id: Unchanged(access.id), //TODO ved ikke om unchanged betyder det jeg tror det betyder
                    role: Default::default(),
                    model_id: Unchanged(access.model_id),
                    user_id: Unchanged(access.user_id),
                }
                .update(&self.db_context.get_connection())
                .await
            }
        };
        return updated_access;
    }

    /// Deletes a access entity by id
    async fn delete(&self, entity_id: i32) -> Result<Access, DbErr> {
        let access = self.get_by_id(entity_id).await?;
        match access {
            None => Err(DbErr::Exec(RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(access) => {
                AccessEntity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(access)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/access_context.rs"]
mod tests;
