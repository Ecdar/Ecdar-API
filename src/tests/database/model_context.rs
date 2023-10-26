#[cfg(test)]
mod database_tests {
    use crate::database::user_context;
    use crate::tests::database::helpers::{helpers::*, self};
    use crate::{
        database:: {
            database_context::DatabaseContext,
            entity_context::EntityContextTrait,
            model_context::{ModelContext, self},
            user_context::UserContext,
        },
        entities::model::{Entity as ModelEntity, Model as ModelModel},
        entities::user::{Entity as UserEntity, Model as UserModel},
    };
    use sea_orm::{DbErr, DatabaseConnection, Schema, DatabaseBackend, sea_query::TableCreateStatement, ConnectionTrait, Database, EntityTrait};

    async fn setup_schema(db: &DatabaseConnection) {
        // Setup Schema helper
        let schema = Schema::new(DatabaseBackend::Sqlite);

        // Derive from Entity
        let stmt: TableCreateStatement = schema.create_table_from_entity(ModelEntity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
        let stmt: TableCreateStatement = schema.create_table_from_entity(UserEntity);
        let _ = db.execute(db.get_database_backend().build(&stmt)).await;
    }

    #[tokio::test]
    async fn create_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(helpers::helpers::setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);

        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        let new_user = UserModel {
            id: 1,
            email: "some@suck.dj".to_owned(),
            username: "ula".to_owned(),
            password: "123".to_owned(),
        };

        // Model to be created
        let new_model = ModelModel {
            id: 1,
            name: "Coffee Machine".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_model.clone()).await?;

        let fetched_model = ModelEntity::find_by_id(created_model.id).one(&model_context.db_context.get_connection()).await?.clone().unwrap();

        assert_eq!(fetched_model.name, created_model.name);

        Ok(())
    }

    #[tokio::test]
    async fn get_model_by_id_test() -> Result<(), DbErr> {
        let db_connection = Database::connect("sqlite::memory:").await.unwrap();
        setup_schema(&db_connection).await;
        let db_context = Box::new(DatabaseContext { db_connection });

        let model_context = ModelContext::new(db_context.clone());
        let user_context = UserContext::new(db_context.clone());

        let new_user = UserModel {
            id: 1,
            email: "some@suck.dj".to_owned(),
            username: "ula".to_owned(),
            password: "123".to_owned(),
        };

        // Model to be created
        let new_model = ModelModel {
            id: 1,
            name: "Coffee Machine".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_model.clone()).await?;

        let fetched_model = model_context.get_by_id(created_model.id).await?.unwrap();
        //let fetched_model = ModelEntity::find_by_id(created_model.id).one(&model_context.db_context.get_connection()).await?.clone().unwrap();

        assert_eq!(fetched_model.id, created_model.id);
        assert_eq!(fetched_model.id, new_model.id);

        Ok(())
    }

    #[tokio::test]
    async fn get_all_models_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(helpers::helpers::setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);

        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        let new_user = UserModel {
            id: 1,
            email: "some@suck.dj".to_owned(),
            username: "ula".to_owned(),
            password: "123".to_owned(),
        };

        // Model to be created
        let new_model = ModelModel {
            id: 1,
            name: "Coffee Machine".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };
        let new_model2 = ModelModel {
            id: 2,
            name: "Coffee Machine2".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };


        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_model).await?;
        let created_model2 = model_context.create(new_model2).await?;

        let fetched_all_models = model_context.get_all().await.unwrap().len();

        assert_eq!(fetched_all_models, 2);

        Ok(())
    }

    #[tokio::test]
    async fn update_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(helpers::helpers::setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);

        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        let new_user = UserModel {
            id: 1,
            email: "some@suck.dj".to_owned(),
            username: "ula".to_owned(),
            password: "123".to_owned(),
        };

        // Model to be created
        let original_model = ModelModel {
            id: 1,
            name: "Coffee Machine".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        // Updated model
        let altered_model = ModelModel {
            name: "Shit Machine".to_owned(),
            ..original_model.clone()
        };

        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(original_model.clone()).await?;

        let altered_model = model_context.update(original_model).await.unwrap();
        let fetched_model = model_context.get_by_id(created_model.id).await?.unwrap();

        assert_eq!(altered_model.name, fetched_model.name);

        Ok(())
    }

    #[tokio::test]
    async fn delete_model_test() -> Result<(), DbErr> {
        // DB setup
        let db_contexts = Box::new(helpers::helpers::setup_db_with_entities(vec![AnyEntity::User, AnyEntity::Model]).await);

        let user_context = UserContext::new(db_contexts.to_owned());
        let model_context = ModelContext::new(db_contexts.to_owned());

        let new_user = UserModel {
            id: 1,
            email: "some@suck.dj".to_owned(),
            username: "ula".to_owned(),
            password: "123".to_owned(),
        };

        // Model to be created
        let new_model = ModelModel {
            id: 1,
            name: "Coffee Machine".to_owned(),
            components_info: "{}".to_owned().parse().unwrap(),
            owner_id: 1,
        };

        let created_user = user_context.create(new_user).await?;
        let created_model = model_context.create(new_model.clone()).await?;

        let deleted_model = model_context.delete(created_model.id).await.unwrap();

        assert_eq!(deleted_model.name, created_model.name);

        Ok(())
    }

}