#[cfg(test)]
mod database_tests {
    use crate::tests::database::helpers::{
        create_accesses, create_models, create_users, setup_db_with_entities, AnyEntity,
    };
    use crate::{
        database::{access_context::AccessContext, entity_context::EntityContextTrait},
        entities::{access, model, sea_orm_active_enums::Role, user},
        to_active_models,
    };
    use sea_orm::{entity::prelude::*, IntoActiveModel};

    async fn seed_db() -> (AccessContext, access::Model, user::Model, model::Model) {
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;

        let access_context = AccessContext::new(db_context.clone());

        let user = create_users(1)[0].clone();
        let model = create_models(1, user.id)[0].clone();
        let access = create_accesses(1, user.id, model.id)[0].clone();

        user::Entity::insert(user.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();
        model::Entity::insert(model.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        (access_context, access, user, model)
    }

    #[tokio::test]
    async fn create_test() {
        let (access_context, access, _, _) = seed_db().await;

        let created_access = access_context.create(access.clone()).await.unwrap();

        let fetched_access = access::Entity::find_by_id(created_access.id)
            .one(&access_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(access, created_access);
        assert_eq!(fetched_access, created_access);
    }

    #[tokio::test]
    async fn create_check_unique_pair_model_id_user_id_test() {
        todo!()
        // SHOULD WORK BUT SQLITE DOES NOT ENFORCE PAIR-WISE UNIQUE CONSTRAINT
        //     let db_context =
        //         setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
        //             .await;
        //     let user_context = UserContext::new(db_context.clone());
        //     let model_context = ModelContext::new(db_context.clone());
        //     let access_context = AccessContext::new(db_context.clone());
        //
        //     let new_user = create_users(1)[0].to_owned();
        //     let new_model = create_models(1, new_user.id)[0].clone();
        //
        //     let new_accesses = create_accesses(3, new_user.id, new_model.id);
        //
        //     user_context.create(new_user).await.unwrap();
        //     model_context.create(new_model).await.unwrap();
        //     let _ = access_context.create(new_accesses[0].clone()).await.unwrap();
        //     let res = access_context.create(new_accesses[1].clone()).await.expect_err("This should not be Ok()");
    }

    #[tokio::test]
    async fn create_auto_increment_test() {
        let (access_context, access, _, _) = seed_db().await;

        let created_access1 = access_context.create(access.clone()).await.unwrap();
        let created_access2 = access_context.create(access.clone()).await.unwrap();

        let fetched_access1 = access::Entity::find_by_id(created_access1.id)
            .one(&access_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        let fetched_access2 = access::Entity::find_by_id(created_access2.id)
            .one(&access_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_ne!(fetched_access1.id, fetched_access2.id);
        assert_ne!(created_access1.id, created_access2.id);
        assert_eq!(created_access1.id, fetched_access1.id);
        assert_eq!(created_access2.id, fetched_access2.id);
    }

    #[tokio::test]
    async fn get_by_id_test() {
        let (access_context, access, _, _) = seed_db().await;

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let fetched_access = access_context.get_by_id(access.id).await.unwrap().unwrap();

        assert_eq!(access, fetched_access);
    }

    #[tokio::test]
    async fn get_by_non_existing_id_test() {
        let (access_context, _, _, _) = seed_db().await;

        let fetched_access = access_context.get_by_id(1).await.unwrap();

        assert!(fetched_access.is_none());
    }

    #[tokio::test]
    async fn get_all_test() {
        let (access_context, _, user, model) = seed_db().await;

        let new_accesses = create_accesses(3, user.id, model.id);

        access::Entity::insert_many(to_active_models!(new_accesses.clone()))
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(access_context.get_all().await.unwrap().len(), 3);

        let mut sorted: Vec<access::Model> = new_accesses.clone();
        sorted.sort_by_key(|k| k.id);

        for (i, access) in sorted.into_iter().enumerate() {
            assert_eq!(access, new_accesses[i]);
        }
    }

    #[tokio::test]
    async fn get_all_empty_test() {
        let (access_context, _, _, _) = seed_db().await;

        let result = access_context.get_all().await.unwrap();
        let empty_accesses: Vec<access::Model> = vec![];

        assert_eq!(empty_accesses, result);
    }

    #[tokio::test]
    async fn update_test() {
        let (access_context, access, _, _) = seed_db().await;

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let new_access = access::Model { ..access };

        let updated_access = access_context.update(new_access.clone()).await.unwrap();

        let fetched_access = access::Entity::find_by_id(updated_access.id)
            .one(&access_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_access, updated_access);
        assert_eq!(updated_access, fetched_access);
    }

    #[tokio::test]
    async fn update_modifies_role_test() {
        let (access_context, access, _, _) = seed_db().await;

        let access = access::Model {
            role: Role::Editor,
            ..access
        };

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let new_access = access::Model {
            role: Role::Commenter,
            ..access
        };

        let updated_access = access_context.update(new_access.clone()).await.unwrap();

        assert_ne!(access, updated_access);
        assert_ne!(access, new_access);
    }

    #[tokio::test]
    async fn update_does_not_modify_id_test() {
        let (access_context, access, _, _) = seed_db().await;
        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_access = access::Model {
            id: &access.id + 1,
            ..access.clone()
        };
        let res = access_context.update(updated_access.clone()).await;

        assert!(matches!(res.unwrap_err(), DbErr::RecordNotUpdated));
    }
    #[tokio::test]
    async fn update_does_not_modify_model_id_test() {
        let (access_context, access, _, _) = seed_db().await;

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_access = access::Model {
            model_id: &access.model_id + 1,
            ..access.clone()
        };
        let res = access_context.update(updated_access.clone()).await.unwrap();

        assert_eq!(access, res);
    }
    #[tokio::test]
    async fn update_does_not_modify_user_id_test() {
        let (access_context, access, _, _) = seed_db().await;

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let updated_access = access::Model {
            user_id: &access.user_id + 1,
            ..access.clone()
        };
        let res = access_context.update(updated_access.clone()).await.unwrap();

        assert_eq!(access, res);
    }

    #[tokio::test]
    async fn update_non_existing_id_test() {
        let (access_context, access, _, _) = seed_db().await;

        let updated_access = access_context.update(access.clone()).await;

        assert!(matches!(
            updated_access.unwrap_err(),
            DbErr::RecordNotUpdated
        ));
    }

    #[tokio::test]
    async fn delete_test() {
        let (access_context, access, _, _) = seed_db().await;

        access::Entity::insert(access.clone().into_active_model())
            .exec(&access_context.db_context.get_connection())
            .await
            .unwrap();

        let deleted_access = access_context.delete(access.id).await.unwrap();

        let all_accesses = access::Entity::find()
            .all(&access_context.db_context.get_connection())
            .await
            .unwrap();

        assert_eq!(access, deleted_access);
        assert!(all_accesses.is_empty());
    }

    #[tokio::test]
    async fn delete_non_existing_id_test() {
        let (access_context, _, _, _) = seed_db().await;

        let deleted_access = access_context.delete(1).await;

        assert!(matches!(
            deleted_access.unwrap_err(),
            DbErr::RecordNotFound(_)
        ));
    }
}
