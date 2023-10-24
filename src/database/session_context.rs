use sea_orm::prelude::async_trait::async_trait;
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};

use crate::database::database_context::DatabaseContext;
use crate::database::entity_context::EntityContextTrait;
use crate::entities::prelude::Session;
use crate::entities::session::{ActiveModel, Model};

pub struct SessionContext {
    db_context: DatabaseContext,
}

#[async_trait]
pub trait SessionContextTrait {}

impl SessionContextTrait for SessionContext {}

#[async_trait]
impl EntityContextTrait<Model> for SessionContext {
    fn new(db_context: DatabaseContext) -> Self {
        SessionContext {
            db_context: db_context,
        }
    }

    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let session = ActiveModel {
            id: Default::default(),
            token: Set(entity.token),
            created_at: Set(entity.created_at),
            user_id: Set(entity.user_id),
        };

        let session = session.insert(&self.db_context.db).await;
        session
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Model>, DbErr> {
        Session::find_by_id(id).one(&self.db_context.db).await
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        Session::find().all(&self.db_context.db).await
    }

    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        let res = &self.get_by_id(entity.id).await?;
        let updated_session: Result<Model, DbErr> = match res {
            None => Err(DbErr::RecordNotFound(String::from(format!(
                "Could not find entity {:?}",
                entity
            )))),
            Some(session) => {
                ActiveModel {
                    id: Unchanged(session.id),
                    token: Set(entity.token),
                    created_at: Set(entity.created_at),
                    user_id: Unchanged(session.user_id), //TODO Should it be allowed to change the user_id of a session?
                }
                .update(&self.db_context.db)
                .await
            }
        };
        return updated_session;
    }

    async fn delete(&self, id: i32) -> Result<Model, DbErr> {
        let session = self.get_by_id(id).await?;
        match session {
            None => Err(DbErr::Exec(sea_orm::RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(session) => {
                Session::delete_by_id(id).exec(&self.db_context.db).await?;
                Ok(session)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/database/session_context.rs"]
mod tests;

