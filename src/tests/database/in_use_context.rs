#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::*;
    use chrono::Utc;
    use sea_orm::{entity::prelude::*, IntoActiveModel};
    use std::matches;
    use std::str::FromStr;

    use crate::database::database_context::DatabaseContextTrait;
    use crate::{
        database::{
            entity_context::EntityContextTrait,
            in_use_context::{DbErr, InUseContext},
        },
        entities::{in_use, model, session, user},
    };

    struct Seed {
        in_use_context: InUseContext,
        models: Vec<model::Model>,
        sessions: Vec<session::Model>,
        users: Vec<user::Model>,
    }

    async fn seed_db() -> Seed {
        let db_context = setup_db_with_entities(vec![
            AnyEntity::Session,
            AnyEntity::Model,
            AnyEntity::User,
            AnyEntity::InUse,
        ])
        .await;

        let users: Vec<user::Model> = create_users(2);
        let user = users[0].clone();

        let mut sessions: Vec<session::Model> = create_sessions(2, user.clone().id);
        sessions[1].user_id = users[1].id;
        sessions[1].token = Uuid::from_str("67e55044-10b1-426f-9247-bb680e5fe0c8".clone()).unwrap();

        let mut models: Vec<model::Model> = create_models(2, user.clone().id);
        models[1].owner_id = users[1].id;

        user::Entity::insert_many(activate!(users.clone(), user::ActiveModel))
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert_many(activate!(models.clone(), model::ActiveModel))
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        session::Entity::insert_many(activate!(sessions.clone(), session::ActiveModel))
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let in_use_context = InUseContext::new(db_context.clone());

        Seed {
            in_use_context,
            models,
            sessions,
            users,
        }
    }

    #[tokio::test]
    async fn create_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        let inserted_in_use = seed.in_use_context.create(in_use.clone()).await.unwrap();

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.clone().model_id)
            .one(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(inserted_in_use, fetched_in_use);
    }

    #[tokio::test]
    async fn create_default_created_at_test() {
        let t_min = Utc::now().timestamp();

        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);

        let mut in_use = in_uses[0].clone();
        let inserted_in_use = seed.in_use_context.create(in_use.clone()).await.unwrap();

        let fetched_in_use = in_use::Entity::find_by_id(inserted_in_use.clone().model_id)
            .one(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let t_max = Utc::now().timestamp();

        let t_actual = fetched_in_use.clone().latest_activity.timestamp();

        assert!(t_min <= t_actual && t_actual <= t_max)
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_in_use = seed
            .in_use_context
            .get_by_id(in_use.model_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(fetched_in_use, in_use)
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        let seed = seed_db().await;

        let in_use = seed.in_use_context.get_by_id(1).await;

        assert!(in_use.unwrap().is_none())
    }

    #[tokio::test]
    async fn get_all_test() {
        let seed = seed_db().await;

        let mut in_uses: Vec<in_use::Model> =
            create_in_use(2, seed.models[0].clone().id, seed.sessions[0].clone().id);
        in_uses[1].model_id = seed.models[1].id;
        in_uses[1].session_id = seed.sessions[1].id;

        in_use::Entity::insert_many(activate!(in_uses.clone(), in_use::ActiveModel))
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_in_use = seed.in_use_context.get_all().await.unwrap();

        assert_eq!(2, fetched_in_use.len())
    }

    #[tokio::test]
    async fn get_all_empty_test() {
        let seed = seed_db().await;

        let in_uses = seed.in_use_context.get_all().await.unwrap();

        assert_eq!(0, in_uses.len())
    }

    #[tokio::test]
    async fn update_latest_activity_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let update_in_use = in_use::Model {
            latest_activity: Utc::now().naive_local(),
            ..in_use.clone()
        };

        let updated_in_use = seed
            .in_use_context
            .update(update_in_use.clone())
            .await
            .unwrap();

        assert_eq!(update_in_use, updated_in_use);
        assert_ne!(in_use, updated_in_use);
    }

    #[tokio::test]
    async fn update_non_existing_test() {
        let seed = seed_db().await;

        let update_in_use = in_use::Model {
            latest_activity: Utc::now().naive_local(),
            model_id: 1,
            session_id: 1,
        };

        let updated_in_use = seed.in_use_context.update(update_in_use.clone()).await;

        assert!(matches!(
            updated_in_use.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn update_model_id_should_fail_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let update_in_use = in_use::Model {
            model_id: 2,
            ..in_use.clone()
        };

        let updated_in_use = seed.in_use_context.update(update_in_use.clone()).await;

        assert!(matches!(
            updated_in_use.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn update_session_id_should_fail_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let update_in_use = in_use::Model {
            session_id: 2,
            ..in_use.clone()
        };

        let updated_in_use = seed.in_use_context.update(update_in_use.clone()).await;
        assert_eq!(updated_in_use.unwrap().session_id, 1);
    }

    #[tokio::test]
    async fn delete_test() {
        let seed = seed_db().await;

        let in_uses: Vec<in_use::Model> =
            create_in_use(1, seed.models[0].clone().id, seed.sessions[0].clone().id);
        let in_use = in_uses[0].clone();

        in_use::Entity::insert(in_use.clone().into_active_model())
            .exec(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_in_use = seed.in_use_context.delete(in_use.model_id).await.unwrap();

        let all_in_uses = in_use::Entity::find()
            .all(&seed.in_use_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(in_use, deleted_in_use);
        assert_eq!(0, all_in_uses.len())
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        let seed = seed_db().await;

        let deleted_in_use = seed.in_use_context.delete(1).await;

        assert!(matches!(
            deleted_in_use.unwrap_err(),
            DbErr::RecordNotFound(_)
        ))
    }
}
