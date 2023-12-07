use crate::tests::database::helpers::*;
use crate::{
    contexts::context_impls::ProjectContext,
    contexts::context_traits::EntityContextTrait,
    entities::{access, in_use, project, query, session, user},
    to_active_models,
};
use sea_orm::error::DbErr;
use sea_orm::{entity::prelude::*, IntoActiveModel};
use std::matches;

async fn seed_db() -> (ProjectContext, project::Model, user::Model) {
    let db_context = get_reset_database_context().await;

    let project_context = ProjectContext::new(db_context);

    let user = create_users(1)[0].clone();
    let project = create_projects(1, user.id)[0].clone();

    user::Entity::insert(user.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    (project_context, project, user)
}

#[tokio::test]
async fn create_test() {
    let (project_context, project, _) = seed_db().await;

    let created_project = project_context.create(project.clone()).await.unwrap();

    let fetched_project = project::Entity::find_by_id(created_project.id)
        .one(&project_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(project, created_project);
    assert_eq!(fetched_project, created_project);
}

#[tokio::test]
async fn create_auto_increment_test() {
    let (project_context, project, _) = seed_db().await;

    let projects = create_projects(2, project.owner_id);

    let created_project1 = project_context.create(projects[0].clone()).await.unwrap();
    let created_project2 = project_context.create(projects[1].clone()).await.unwrap();

    let fetched_project1 = project::Entity::find_by_id(created_project1.id)
        .one(&project_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    let fetched_project2 = project::Entity::find_by_id(created_project2.id)
        .one(&project_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_ne!(fetched_project1.id, fetched_project2.id);
    assert_ne!(created_project1.id, created_project2.id);
    assert_eq!(created_project1.id, fetched_project1.id);
    assert_eq!(created_project2.id, fetched_project2.id);
}

#[tokio::test]
async fn get_by_id_test() {
    let (project_context, project, _) = seed_db().await;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let fetched_project = project_context
        .get_by_id(project.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(project, fetched_project);
}

#[tokio::test]
async fn get_by_non_existing_id_test() {
    let (project_context, _, _) = seed_db().await;

    let fetched_project = project_context.get_by_id(1).await.unwrap();

    assert!(fetched_project.is_none());
}

#[tokio::test]
async fn get_all_test() {
    let (project_context, _, user) = seed_db().await;

    let new_projects = create_projects(3, user.id);

    project::Entity::insert_many(to_active_models!(new_projects.clone()))
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(project_context.get_all().await.unwrap().len(), 3);

    let mut sorted = new_projects.clone();
    sorted.sort_by_key(|k| k.id);

    for (i, project) in sorted.into_iter().enumerate() {
        assert_eq!(project, new_projects[i]);
    }
}

#[tokio::test]
async fn get_all_empty_test() {
    let (project_context, _, _) = seed_db().await;

    let result = project_context.get_all().await.unwrap();
    let empty_projects: Vec<project::Model> = vec![];

    assert_eq!(empty_projects, result);
}

#[tokio::test]
async fn update_test() {
    let (project_context, project, _) = seed_db().await;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model { ..project };

    let updated_project = project_context.update(new_project.clone()).await.unwrap();

    let fetched_project = project::Entity::find_by_id(updated_project.id)
        .one(&project_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(new_project, updated_project);
    assert_eq!(updated_project, fetched_project);
}

#[tokio::test]
async fn update_modifies_name_test() {
    let (project_context, project, _) = seed_db().await;

    let project = project::Model {
        name: "project1".into(),
        ..project.clone()
    };

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model {
        name: "project2".into(),
        ..project.clone()
    };

    let updated_project = project_context.update(new_project.clone()).await.unwrap();

    assert_ne!(project, updated_project);
    assert_ne!(project, new_project);
}

#[tokio::test]
async fn update_modifies_components_info_test() {
    let (project_context, project, _) = seed_db().await;

    let project = project::Model {
        components_info: "{\"a\":1}".to_owned().parse().unwrap(),
        ..project.clone()
    };

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model {
        components_info: "{\"a\":2}".to_owned().parse().unwrap(),
        ..project.clone()
    };

    let updated_project = project_context.update(new_project.clone()).await.unwrap();

    assert_ne!(project, updated_project);
    assert_ne!(project, new_project);
}

#[tokio::test]
async fn update_does_not_modify_id_test() {
    let (project_context, project, _) = seed_db().await;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model {
        id: &project.id + 1,
        ..project.clone()
    };

    let res = project_context.update(new_project.clone()).await;

    assert!(matches!(res.unwrap_err(), DbErr::RecordNotUpdated));
}

#[tokio::test]
async fn update_does_not_modify_owner_id_test() {
    let (project_context, project, _) = seed_db().await;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model {
        owner_id: &project.owner_id + 1,
        ..project.clone()
    };

    let res = project_context.update(new_project.clone()).await.unwrap();

    assert_eq!(project, res);
}

#[tokio::test]
async fn update_check_query_outdated_test() {
    let (project_context, project, _) = seed_db().await;

    let mut query = create_queries(1, project.id)[0].clone();

    query.outdated = false;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    query::Entity::insert(query.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let new_project = project::Model { ..project };

    let updated_project = project_context.update(new_project.clone()).await.unwrap();

    let fetched_query = query::Entity::find_by_id(updated_project.id)
        .one(&project_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert!(fetched_query.outdated);
}

#[tokio::test]
async fn update_non_existing_id_test() {
    let (project_context, project, _) = seed_db().await;

    let updated_project = project_context.update(project.clone()).await;

    assert!(matches!(
        updated_project.unwrap_err(),
        DbErr::RecordNotUpdated
    ));
}

#[tokio::test]
async fn delete_test() {
    // Setting up contexts and user context
    let (project_context, project, _) = seed_db().await;

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    let deleted_project = project_context.delete(project.id).await.unwrap();

    let all_projects = project::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(project, deleted_project);
    assert_eq!(all_projects.len(), 0);
}

#[tokio::test]
async fn delete_cascade_query_test() {
    let (project_context, project, _) = seed_db().await;

    let query = create_queries(1, project.clone().id)[0].clone();

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();
    query::Entity::insert(query.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    project_context.delete(project.id).await.unwrap();

    let all_queries = query::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();
    let all_projects = project::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_queries.len(), 0);
    assert_eq!(all_projects.len(), 0);
}

#[tokio::test]
async fn delete_cascade_access_test() {
    let (project_context, project, _) = seed_db().await;

    let access = create_accesses(1, 1, project.clone().id)[0].clone();

    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();
    access::Entity::insert(access.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    project_context.delete(project.id).await.unwrap();

    let all_projects = project::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();
    let all_accesses = access::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_projects.len(), 0);
    assert_eq!(all_accesses.len(), 0);
}

#[tokio::test]
async fn delete_cascade_in_use_test() {
    let (project_context, project, user) = seed_db().await;

    let session = create_sessions(1, user.clone().id)[0].clone();
    let in_use = create_in_uses(1, project.clone().id, 1)[0].clone();

    session::Entity::insert(session.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();
    project::Entity::insert(project.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();
    in_use::Entity::insert(in_use.clone().into_active_model())
        .exec(&project_context.db_context.get_connection())
        .await
        .unwrap();

    project_context.delete(project.id).await.unwrap();

    let all_projects = project::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();
    let all_in_uses = in_use::Entity::find()
        .all(&project_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_projects.len(), 0);
    assert_eq!(all_in_uses.len(), 0);
}

#[tokio::test]
async fn delete_non_existing_id_test() {
    let (project_context, _, _) = seed_db().await;

    let deleted_project = project_context.delete(1).await;

    assert!(matches!(
        deleted_project.unwrap_err(),
        DbErr::RecordNotFound(_)
    ));
}
