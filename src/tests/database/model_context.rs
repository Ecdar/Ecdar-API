use crate::tests::database::helpers::*;
use crate::{
    database::{entity_context::EntityContextTrait, project_context::ProjectContext},
    entities::{access, in_use, model, query, session, user},
    to_active_models,
};
use sea_orm::error::DbErr;
use sea_orm::{entity::prelude::*, IntoActiveModel};
use std::matches;

async fn seed_db() -> (ProjectContext, model::Model, user::Model) {
    let db_context = get_reset_database_context().await;

    let model_context = ProjectContext::new(db_context);

    let user = create_users(1)[0].clone();
    let model = create_models(1, user.id)[0].clone();

    user::Entity::insert(user.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    (model_context, model, user)
}

#[tokio::test]
async fn create_test() {
    let (model_context, model, _) = seed_db().await;

    let created_model = model_context.create(model.clone()).await.unwrap();

    let fetched_model = model::Entity::find_by_id(created_model.id)
        .one(&model_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(model, created_model);
    assert_eq!(fetched_model, created_model);
}

#[tokio::test]
async fn create_auto_increment_test() {
    let (model_context, model, _) = seed_db().await;

    let models = create_models(2, model.owner_id);

    let created_model1 = model_context.create(models[0].clone()).await.unwrap();
    let created_model2 = model_context.create(models[1].clone()).await.unwrap();

    let fetched_model1 = model::Entity::find_by_id(created_model1.id)
        .one(&model_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    let fetched_model2 = model::Entity::find_by_id(created_model2.id)
        .one(&model_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_ne!(fetched_model1.id, fetched_model2.id);
    assert_ne!(created_model1.id, created_model2.id);
    assert_eq!(created_model1.id, fetched_model1.id);
    assert_eq!(created_model2.id, fetched_model2.id);
}

#[tokio::test]
async fn get_by_id_test() {
    let (model_context, model, _) = seed_db().await;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let fetched_model = model_context.get_by_id(model.id).await.unwrap().unwrap();

    assert_eq!(model, fetched_model);
}

#[tokio::test]
async fn get_by_non_existing_id_test() {
    let (model_context, _, _) = seed_db().await;

    let fetched_model = model_context.get_by_id(1).await.unwrap();

    assert!(fetched_model.is_none());
}

#[tokio::test]
async fn get_all_test() {
    let (model_context, _, user) = seed_db().await;

    let new_models = create_models(3, user.id);

    model::Entity::insert_many(to_active_models!(new_models.clone()))
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(model_context.get_all().await.unwrap().len(), 3);

    let mut sorted = new_models.clone();
    sorted.sort_by_key(|k| k.id);

    for (i, model) in sorted.into_iter().enumerate() {
        assert_eq!(model, new_models[i]);
    }
}

#[tokio::test]
async fn get_all_empty_test() {
    let (model_context, _, _) = seed_db().await;

    let result = model_context.get_all().await.unwrap();
    let empty_models: Vec<model::Model> = vec![];

    assert_eq!(empty_models, result);
}

#[tokio::test]
async fn update_test() {
    let (model_context, model, _) = seed_db().await;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model { ..model };

    let updated_model = model_context.update(new_model.clone()).await.unwrap();

    let fetched_model = model::Entity::find_by_id(updated_model.id)
        .one(&model_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(new_model, updated_model);
    assert_eq!(updated_model, fetched_model);
}

#[tokio::test]
async fn update_modifies_name_test() {
    let (model_context, model, _) = seed_db().await;

    let model = model::Model {
        name: "model1".into(),
        ..model.clone()
    };

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model {
        name: "model2".into(),
        ..model.clone()
    };

    let updated_model = model_context.update(new_model.clone()).await.unwrap();

    assert_ne!(model, updated_model);
    assert_ne!(model, new_model);
}

#[tokio::test]
async fn update_modifies_components_info_test() {
    let (model_context, model, _) = seed_db().await;

    let model = model::Model {
        components_info: "{\"a\":1}".to_owned().parse().unwrap(),
        ..model.clone()
    };

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model {
        components_info: "{\"a\":2}".to_owned().parse().unwrap(),
        ..model.clone()
    };

    let updated_model = model_context.update(new_model.clone()).await.unwrap();

    assert_ne!(model, updated_model);
    assert_ne!(model, new_model);
}

#[tokio::test]
async fn update_does_not_modify_id_test() {
    let (model_context, model, _) = seed_db().await;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model {
        id: &model.id + 1,
        ..model.clone()
    };

    let res = model_context.update(new_model.clone()).await;

    assert!(matches!(res.unwrap_err(), DbErr::RecordNotUpdated));
}

#[tokio::test]
async fn update_does_not_modify_owner_id_test() {
    let (model_context, model, _) = seed_db().await;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model {
        owner_id: &model.owner_id + 1,
        ..model.clone()
    };

    let res = model_context.update(new_model.clone()).await.unwrap();

    assert_eq!(model, res);
}

#[tokio::test]
async fn update_check_query_outdated_test() {
    let (model_context, model, _) = seed_db().await;

    let mut query = create_queries(1, model.id)[0].clone();

    query.outdated = false;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    query::Entity::insert(query.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let new_model = model::Model { ..model };

    let updated_model = model_context.update(new_model.clone()).await.unwrap();

    let fetched_query = query::Entity::find_by_id(updated_model.id)
        .one(&model_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert!(fetched_query.outdated);
}

#[tokio::test]
async fn update_non_existing_id_test() {
    let (model_context, model, _) = seed_db().await;

    let updated_model = model_context.update(model.clone()).await;

    assert!(matches!(
        updated_model.unwrap_err(),
        DbErr::RecordNotUpdated
    ));
}

#[tokio::test]
async fn delete_test() {
    // Setting up database and user context
    let (model_context, model, _) = seed_db().await;

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    let deleted_model = model_context.delete(model.id).await.unwrap();

    let all_models = model::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(model, deleted_model);
    assert_eq!(all_models.len(), 0);
}

#[tokio::test]
async fn delete_cascade_query_test() {
    let (model_context, model, _) = seed_db().await;

    let query = create_queries(1, model.clone().id)[0].clone();

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();
    query::Entity::insert(query.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    model_context.delete(model.id).await.unwrap();

    let all_queries = query::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();
    let all_models = model::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_queries.len(), 0);
    assert_eq!(all_models.len(), 0);
}

#[tokio::test]
async fn delete_cascade_access_test() {
    let (model_context, model, _) = seed_db().await;

    let access = create_accesses(1, 1, model.clone().id)[0].clone();

    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();
    access::Entity::insert(access.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    model_context.delete(model.id).await.unwrap();

    let all_models = model::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();
    let all_accesses = access::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_models.len(), 0);
    assert_eq!(all_accesses.len(), 0);
}

#[tokio::test]
async fn delete_cascade_in_use_test() {
    let (model_context, model, user) = seed_db().await;

    let session = create_sessions(1, user.clone().id)[0].clone();
    let in_use = create_in_uses(1, model.clone().id, 1)[0].clone();

    session::Entity::insert(session.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();
    model::Entity::insert(model.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();
    in_use::Entity::insert(in_use.clone().into_active_model())
        .exec(&model_context.db_context.get_connection())
        .await
        .unwrap();

    model_context.delete(model.id).await.unwrap();

    let all_models = model::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();
    let all_in_uses = in_use::Entity::find()
        .all(&model_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_models.len(), 0);
    assert_eq!(all_in_uses.len(), 0);
}

#[tokio::test]
async fn delete_non_existing_id_test() {
    let (model_context, _, _) = seed_db().await;

    let deleted_model = model_context.delete(1).await;

    assert!(matches!(
        deleted_model.unwrap_err(),
        DbErr::RecordNotFound(_)
    ));
}
