use crate::api::server::protobuf::AccessInfo;
use crate::contexts::context_traits::EntityContextTrait;
use crate::entities::access;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait AccessContextTrait: EntityContextTrait<access::Model> {
    /// Searches for an access entity by `User` and `Project` id, 
    /// returning [`Some`] if any entity was found, [`None`] otherwise
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn get_access_by_uid_and_project_id(
        &self,
        uid: i32,
        project_id: i32,
    ) -> Result<Option<access::Model>, DbErr>;
    /// Returns all [`access::Model`] that are associated with a given `Project``
    async fn get_access_by_project_id(&self, project_id: i32) -> Result<Vec<AccessInfo>, DbErr>;
}
