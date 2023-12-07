use crate::api::server::server::AccessInfo;
use crate::contexts::context_traits::{
    AccessContextTrait, DatabaseContextTrait, EntityContextTrait,
};
use crate::entities::access;
use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, DbErr, EntityTrait, QueryFilter};
use std::sync::Arc;

pub struct AccessContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
impl AccessContextTrait for AccessContext {
    async fn get_access_by_uid_and_project_id(
        &self,
        uid: i32,
        project_id: i32,
    ) -> Result<Option<access::Model>, DbErr> {
        access::Entity::find()
            .filter(
                Condition::all()
                    .add(access::Column::UserId.eq(uid))
                    .add(access::Column::ProjectId.eq(project_id)),
            )
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_access_by_project_id(&self, project_id: i32) -> Result<Vec<AccessInfo>, DbErr> {
        access::Entity::find()
            .filter(access::Column::ProjectId.eq(project_id))
            .into_model::<AccessInfo>()
            .all(&self.db_context.get_connection())
            .await
    }
}

impl AccessContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> AccessContext {
        AccessContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<access::Model> for AccessContext {
    /// Used for creating an access::Model entity
    /// # Example
    /// ```
    /// let access = access::Model {
    ///     id: Default::default(),
    ///     role: Role::Editor,
    ///     user_id: 1,
    ///     project_id: 1
    /// };
    /// let context : AccessContext = AccessContext::new(...);
    /// context.create(model);
    /// ```
    async fn create(&self, entity: access::Model) -> Result<access::Model, DbErr> {
        let access = access::ActiveModel {
            id: Default::default(),
            role: Set(entity.role),
            project_id: Set(entity.project_id),
            user_id: Set(entity.user_id),
        };
        let access: access::Model = access.insert(&self.db_context.get_connection()).await?;
        Ok(access)
    }

    /// Returns a single access entity (uses primary key)
    /// # Example
    /// ```
    /// let context : AccessContext = AccessContext::new(...);
    /// let model : Model = context.get_by_id(1).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<access::Model>, DbErr> {
        access::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns all the access entities
    /// # Example
    /// ```
    /// let context : AccessContext = AccessContext::new(...);
    /// let model : vec<Model> = context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<access::Model>, DbErr> {
        access::Entity::find()
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
    ///     project_id: access.project_id
    /// }
    /// ```
    /// # Note
    /// The access entity's ids will never be changed. If this behavior is wanted, delete the old access and create a new one.
    async fn update(&self, entity: access::Model) -> Result<access::Model, DbErr> {
        access::ActiveModel {
            id: Unchanged(entity.id),
            role: Set(entity.role),
            project_id: Unchanged(entity.project_id),
            user_id: Unchanged(entity.user_id),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    /// Deletes a access entity by id
    async fn delete(&self, entity_id: i32) -> Result<access::Model, DbErr> {
        let access = self.get_by_id(entity_id).await?;
        match access {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(access) => {
                access::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(access)
            }
        }
    }
}
#[cfg(test)]
#[path = "../../tests/contexts/access_context.rs"]
mod access_context_tests;
