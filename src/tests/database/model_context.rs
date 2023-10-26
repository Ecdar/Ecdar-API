#[cfg(test)]
mod database_tests {
    use crate::database::database_context::DatabaseContextTrait;
    use crate::database::query_context::QueryContext;
    use crate::database::user_context;
    use crate::tests::database::helpers::{self, setup_db_with_entities, AnyEntity};
    use crate::{
        database:: {
            database_context::DatabaseContext,
            entity_context::EntityContextTrait,
            model_context::{ModelContext, self},
            user_context::UserContext,
        },
        entities::model::{Entity as ModelEntity, Model as ModelModel},
        entities::user::{Entity as UserEntity, Model as UserModel},
        entities::query::{Model as QueryModel},
    };
    use sea_orm::IntoActiveModel;
    use sea_orm::{DbErr, DatabaseConnection, Schema, DatabaseBackend, sea_query::TableCreateStatement, ConnectionTrait, Database, EntityTrait};

    fn user_template() -> UserModel {
        UserModel {
            id: 1,
            email: "some@suck.cum".to_owned(),
            username: "Long".to_owned(),
            password: "Johnson".to_owned(),
        }
    }

    fn model_template() -> Vec<ModelModel> {
        vec![
            ModelModel {
                id: Default::default(),
                name: "Coffee Machine".to_owned(),
                components_info: "{}".to_owned().parse().unwrap(),
                owner_id: 1,
            },
            ModelModel {
                id: Default::default(),
                name: "Coffee Machine No.2".to_owned(),
                components_info: "{}".to_owned().parse().unwrap(),
                owner_id: 1,
            }
        ]
    }
    
    #[tokio::test]
    async fn create_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_model = model_template();

        // Create user and model
        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_model[0].clone()).await?;

        // Fetch model from database
        let fetched_model = ModelEntity::find_by_id(created_model.id).one(&model_context.db_context.get_connection()).await?.unwrap();

        assert_eq!(fetched_model.name, created_model.name);

        Ok(())
    }

    #[tokio::test]
    async fn create_model_id_auto_increment_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_model = model_template();

        // Create user and model
        let created_user = user_context.create(new_user).await?;
        let created_model1 = model_context.create(new_model[0].clone()).await?;
        let created_model2 = model_context.create(new_model[1].clone()).await?;

        // Fetch model from database
        let fetched_model1 = ModelEntity::find_by_id(created_model1.id).one(&model_context.db_context.get_connection()).await?.unwrap();
        let fetched_model2 = ModelEntity::find_by_id(created_model2.id).one(&model_context.db_context.get_connection()).await?.unwrap();

        assert_ne!(fetched_model1.id, fetched_model2.id);
        assert_eq!(fetched_model2.id, 2);

        Ok(())
    }

    #[tokio::test]
    async fn get_model_by_id_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_model = model_template();

        // Create user and model
        //let created_user = user_context.create(new_user).await?;
        let created_user = UserEntity::insert(new_user.into_active_model()).exec(&db_contexts.get_connection()).await.unwrap();
        let created_model = ModelEntity::insert(new_model[0].clone().into_active_model()).exec(&db_contexts.get_connection()).await.unwrap();

        // Fetch model from database
        let fetched_model = model_context.get_by_id(new_model[0].id).await?.unwrap();

        assert_eq!(fetched_model.id, new_model[0].id);

        Ok(())
    }

    #[tokio::test]
    async fn get_all_models_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_models = model_template();

        // Create user and models
        let created_user = UserEntity::insert(new_user.into_active_model()).exec(&db_contexts.get_connection()).await.unwrap();
        //let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_models[0].clone()).await?;
        let created_model2 = model_context.create(new_models[1].clone()).await?;

        // Fetch models from database
        let fetched_all_models = model_context.get_all().await.unwrap().len();

        assert_eq!(fetched_all_models, 2, "models: {}, expected models: {}", fetched_all_models, 2);

        Ok(())
    }

    #[tokio::test]
    async fn update_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model, AnyEntity::Query]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());
        let query_context = QueryContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_models = model_template();
        /*
        let new_query = QueryModel {
            id: 1,
            model_id: 0,
            string: "huhuh".to_owned(),
            result: "{}".to_owned().parse().unwrap(),
            out_dated: false,
        };
        */

        // Updated model
        let altered_model = ModelModel {
            name: "Shit Machine".to_owned(),
            ..new_models[0].clone()
        };

        // Create user and model
        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_models[0].clone()).await?;

        // Update and fetch model
        let altered_model = model_context.update(created_model.clone()).await.unwrap();
        let fetched_model = model_context.get_by_id(created_model.id).await?.unwrap();

        assert_eq!(altered_model.name, fetched_model.name);
        assert_eq!(altered_model.id, fetched_model.id);
        assert_eq!(altered_model.owner_id, fetched_model.owner_id);

        Ok(())
    }

    #[tokio::test]
    async fn delete_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);
        // Setup contexts
        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        // Define user and models
        let new_user = user_template();
        let new_models = model_template();

        // Create user and model
        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_models[0].clone()).await?;

        // Delete model from database
        let deleted_model = model_context.delete(created_model.id).await.unwrap();

        assert_eq!(deleted_model.name, created_model.name);

        Ok(())
    }

}