#[cfg(test)]
mod database_tests {
    use crate::database::database_context::DatabaseContextTrait;
    use crate::tests::database::helpers::{
        create_accesses, create_models, create_users, setup_db_with_entities, AnyEntity,
    };
    use crate::{
        database::{
            access_context::AccessContext, entity_context::EntityContextTrait,
            model_context::ModelContext, user_context::UserContext,
        },
        entities::{access, model, sea_orm_active_enums::Role, user},
        to_active_models,
    };
    use sea_orm::{entity::prelude::*, IntoActiveModel};

    // Test the functionality of the 'create' function, which creates a access in the database
    #[tokio::test]
    async fn create_test() {
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;

        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = create_users(1)[0].clone();
        let new_model = create_models(1, new_user.id)[0].clone();
        let new_access = create_accesses(1, new_user.id, new_model.id)[0].clone();

        // Creates the access in the database using the 'create' function
        user_context.create(new_user).await.unwrap();
        model_context.create(new_model).await.unwrap();
        let created_access = access_context.create(new_access).await.unwrap();

        let fetched_access = access::Entity::find_by_id(created_access.id)
            .one(&access_context.db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        // Assert if the fetched access is the same as the created access
        assert_eq!(fetched_access, created_access);
    }

    #[tokio::test]
    async fn get_by_id_test() -> () {
        // Setting up a sqlite database in memory to test on
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;
        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = create_users(1)[0].to_owned();
        let new_model = create_models(1, new_user.id)[0].clone();

        // Creates a model of the access which will be created
        let new_access = create_accesses(1, new_user.id, new_model.id)[0].clone();

        // Creates the access in the database using the 'create' function
        user_context.create(new_user).await.unwrap();
        model_context.create(new_model).await.unwrap();
        let created_access = access_context.create(new_access).await.unwrap();

        // Fetches the access created using the 'get_by_id' function
        let fetched_access = access_context
            .get_by_id(created_access.id)
            .await
            .unwrap()
            .clone()
            .unwrap();

        // Assert if the fetched access is the same as the created access
        assert_eq!(fetched_access, created_access);
    }
    #[tokio::test]
    async fn get_all_test() -> () {
        // Setting up a sqlite database in memory to test on
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;
        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = create_users(1)[0].to_owned();
        let new_model = create_models(1, new_user.id)[0].clone();

        // Creates a model of the access which will be created
        let new_accesses = create_accesses(3, new_user.id, new_model.id);

        // Creates the access in the database using the 'create' function
        user_context.create(new_user).await.unwrap();
        model_context.create(new_model).await.unwrap();
        access::Entity::insert_many(to_active_models!(new_accesses))
            .exec(&db_context.get_connection())
            .await
            .unwrap();
        assert_eq!(access_context.get_all().await.unwrap().len(), 3)
    }

    #[tokio::test]
    async fn update_test() -> () {
        let db_context =
            setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Access])
                .await;
        let user_context = UserContext::new(db_context.clone());
        let model_context = ModelContext::new(db_context.clone());
        let access_context = AccessContext::new(db_context.clone());

        let new_user = create_users(1)[0].clone();
        let new_model = create_models(1, new_user.id)[0].clone();
        let new_access = create_accesses(1, new_user.id, new_model.id)[0].clone();

        user::Entity::insert(new_user.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        model::Entity::insert(new_model.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        access::Entity::insert(new_access.clone().into_active_model())
            .exec(&db_context.get_connection())
            .await
            .unwrap();

        let updated_access = access::Model {
            role: Role::Commenter,
            ..new_access
        };

        let updated_access = access_context.update(updated_access.clone()).await.unwrap();

        let fetched_access = access::Entity::find_by_id(updated_access.id)
            .one(&db_context.get_connection())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(new_access, updated_access);
        assert_eq!(updated_access, fetched_access);
    }
    // SHOULD WORK BUT SQLITE DOES NOT ENFORCE PAIR-WISE UNIQUE CONSTRAINT
    // #[tokio::test]
    // async fn unique_model_id_and_user_id_constraint_violation_test() -> () {
    //     // Setting up a sqlite database in memory to test on
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
    //     // Creates a model of the access which will be created
    //     let new_accesses = create_accesses(3, new_user.id, new_model.id);
    //
    //     // Creates the access in the database using the 'create' function
    //     user_context.create(new_user).await.unwrap();
    //     model_context.create(new_model).await.unwrap();
    //     let _ = access_context.create(new_accesses[0].clone()).await.unwrap(); // should work
    //     let res = access_context.create(new_accesses[1].clone()).await.expect_err("This should not be Ok()");
    // }
}
