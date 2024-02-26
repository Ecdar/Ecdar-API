use crate::contexts::{db_centexts::DatabaseContextTrait, EntityContextTrait};
use crate::entities::in_use;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, Set, Unchanged};
use std::sync::Arc;

pub trait InUseContextTrait: EntityContextTrait<in_use::Model> {}

pub struct InUseContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

impl InUseContextTrait for InUseContext {}

impl InUseContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> InUseContext {
        InUseContext { db_context }
    }
}
#[async_trait]
impl EntityContextTrait<in_use::Model> for InUseContext {
    /// Used for creating a Model entity
    async fn create(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        let in_use = in_use::ActiveModel {
            project_id: Set(entity.project_id),
            session_id: Set(entity.session_id),
            latest_activity: Set(Utc::now().naive_local()),
        };
        let in_use: in_use::Model = in_use.insert(&self.db_context.get_connection()).await?;
        Ok(in_use)
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<in_use::Model>, DbErr> {
        in_use::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_all(&self) -> Result<Vec<in_use::Model>, DbErr> {
        in_use::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    async fn update(&self, entity: in_use::Model) -> Result<in_use::Model, DbErr> {
        in_use::ActiveModel {
            project_id: Unchanged(entity.project_id),
            session_id: Set(entity.session_id),
            latest_activity: Set(entity.latest_activity),
        }
        .update(&self.db_context.get_connection())
        .await
    }

    async fn delete(&self, entity_id: i32) -> Result<in_use::Model, DbErr> {
        let in_use = self.get_by_id(entity_id).await?;
        match in_use {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(in_use) => {
                in_use::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(in_use)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::helpers::*;
    use crate::{
        contexts::EntityContextTrait,
        contexts::InUseContext,
        entities::{in_use, project, session, user},
        to_active_models,
    };
    use chrono::{Duration, Utc};
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;
    use std::ops::Add;

    async fn seed_db() -> (
        InUseContext,
        in_use::Model,
        session::Model,
        project::Model,
        user::Model,
    ) {
        let db_context = get_reset_database_context().await;

        let in_use_context = InUseContext::new(db_context);

        let user = create_users(1)[0].clone();
        let project = create_projects(1, user.id)[0].clone();
        let session = create_sessions(1, user.id)[0].clone();
        let in_use = create_in_uses(1, project.id, session.id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();
        project::Entity::insert(project.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert(session.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        (in_use_context, in_use, session, project, user)
    }

    #[tokio::test]
    async fn create_test() {
        let (in_use_context, mut in_use, _, _, _) = seed_db().await;

        let inserted_in_use = in_use_context.create(in_use.clone()).await.unwrap();

        in_use.latest_activity = inserted_in_use.latest_activity;

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.clone().project_id)
            .one(&in_use_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(in_use, inserted_in_use);
        assert_eq!(in_use, fetched_in_use);
    }

    #[tokio::test]
    async fn create_default_latest_activity_test() {
        let t_min = Utc::now().timestamp();

        let (in_use_context, in_use, _, _, _) = seed_db().await;

        let inserted_in_use = in_use_context.create(in_use.clone()).await.unwrap();

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.project_id)
            .one(&in_use_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let t_max = Utc::now().timestamp();

        let t_actual = fetched_in_use.clone().latest_activity.timestamp();

        assert!(t_min <= t_actual && t_actual <= t_max)
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_in_use = in_use_context
            .get_by_id(in_use.project_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_in_use, in_use)
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        let (in_use_context, _in_use, _, _, _) = seed_db().await;

        let in_use = in_use_context.get_by_id(1).await;

        assert!(in_use.unwrap().is_none())
    }

    #[tokio::test]
    async fn get_all_test() {
        let (in_use_context, _in_use, session, project, _user) = seed_db().await;

        let in_uses = create_in_uses(1, project.id, session.id);

        in_use::Entity::insert_many(to_active_models!(in_uses.clone()))
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(in_use_context.get_all().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn get_all_empty_test() {
        let (in_use_context, _, _, _, _) = seed_db().await;

        let in_uses = in_use_context.get_all().await.unwrap();

        assert_eq!(0, in_uses.len())
    }

    #[tokio::test]
    async fn update_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let new_in_use = in_use::Model { ..in_use };

        let updated_in_use = in_use_context.update(new_in_use.clone()).await.unwrap();

        let fetched_in_use = in_use::Entity::find_by_id(updated_in_use.project_id)
            .one(&in_use_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_in_use, updated_in_use);
        assert_eq!(updated_in_use, fetched_in_use);
    }

    #[tokio::test]
    async fn update_modifies_latest_activity_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let new_in_use = in_use::Model {
            latest_activity: in_use.clone().latest_activity.add(Duration::seconds(1)),
            ..in_use
        };

        let updated_in_use = in_use_context.update(new_in_use.clone()).await.unwrap();

        assert_ne!(in_use, updated_in_use);
        assert_ne!(in_use, new_in_use);
    }

    #[tokio::test]
    async fn update_modifies_session_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let mut session2 = create_sessions(1, in_use.session_id)[0].clone();
        session2.id = in_use.session_id + 1;
        session2.refresh_token = "new_refresh_token".to_string();
        session2.access_token = "new_access_token".to_string();

        session::Entity::insert(session2.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let new_in_use = in_use::Model {
            session_id: in_use.session_id + 1,
            ..in_use
        };

        let updated_in_use = in_use_context.update(new_in_use.clone()).await.unwrap();

        assert_ne!(in_use, updated_in_use);
        assert_ne!(in_use, new_in_use);
    }

    #[tokio::test]
    async fn update_does_not_modify_project_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_in_use = in_use::Model {
            project_id: in_use.project_id + 1,
            ..in_use.clone()
        };

        let updated_in_use = in_use_context.update(updated_in_use.clone()).await;

        assert!(matches!(
            updated_in_use.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn update_non_existing_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        let updated_in_use = in_use_context.update(in_use.clone()).await;

        assert!(matches!(
            updated_in_use.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn delete_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_in_use = in_use_context.delete(in_use.project_id).await.unwrap();

        let all_in_uses = in_use::Entity::find()
            .all(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(in_use, deleted_in_use);
        assert!(all_in_uses.is_empty());
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        let (in_use_context, _, _, _, _) = seed_db().await;

        let deleted_in_use = in_use_context.delete(1).await;

        assert!(matches!(
            deleted_in_use.unwrap_err(),
            DbErr::RecordNotFound(_)
        ))
    }
}
