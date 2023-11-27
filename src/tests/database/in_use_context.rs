#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use crate::{
        database::{
            entity_context::EntityContextTrait,
            in_use_context::{DbErr, InUseContext},
        },
        entities::{in_use, model, session, user},
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
        model::Model,
        user::Model,
    ) {
        let db_context = get_reset_database_context().await;

        let in_use_context = InUseContext::new(db_context);

        let user = create_users(1)[0].clone();
        let model = create_models(1, user.id)[0].clone();
        let session = create_sessions(1, user.id)[0].clone();
        let in_use = create_in_uses(1, model.id, session.id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert(session.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        (in_use_context, in_use, session, model, user)
    }

    #[tokio::test]
    async fn create_test() {
        let (in_use_context, mut in_use, _, _, _) = seed_db().await;

        let inserted_in_use = in_use_context.create(in_use.clone()).await.unwrap();

        in_use.latest_activity = inserted_in_use.latest_activity;

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.clone().model_id)
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

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.model_id)
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
            .get_by_id(in_use.model_id)
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
        let (in_use_context, _in_use, session, model, user) = seed_db().await;

        let in_uses = create_in_uses(1, model.id, session.id);

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

        let fetched_in_use = in_use::Entity::find_by_id(updated_in_use.model_id)
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
    async fn update_does_not_modify_model_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_in_use = in_use::Model {
            model_id: in_use.model_id + 1,
            ..in_use.clone()
        };

        let updated_in_use = in_use_context.update(updated_in_use.clone()).await;

        assert!(matches!(
            updated_in_use.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn update_does_not_modify_session_id_test() {
        let (in_use_context, in_use, _, _, _) = seed_db().await;

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_in_use = in_use::Model {
            session_id: in_use.session_id + 1,
            ..in_use.clone()
        };

        let updated_in_use = in_use_context.update(updated_in_use.clone()).await.unwrap();
        assert_eq!(in_use, updated_in_use);
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

        let deleted_in_use = in_use_context.delete(in_use.model_id).await.unwrap();

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
