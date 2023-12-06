use crate::{api::auth::TokenType, tests::database::helpers::*};
use sea_orm::{entity::prelude::*, IntoActiveModel};
use std::ops::Add;

use crate::{
    database::{entity_context::EntityContextTrait, session_context::SessionContext},
    entities::{in_use, model, session, user},
    to_active_models,
};

use crate::database::session_context::SessionContextTrait;
use chrono::{Duration, Utc};

async fn seed_db() -> (SessionContext, session::Model, user::Model, model::Model) {
    let db_context = get_reset_database_context().await;

    let session_context = SessionContext::new(db_context);

    let user = create_users(1)[0].clone();
    let model = create_models(1, user.id)[0].clone();
    let session = create_sessions(1, user.id)[0].clone();

    user::Entity::insert(user.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();
    model::Entity::insert(model.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    (session_context, session, user, model)
}

#[tokio::test]
async fn create_test() {
    // Setting up a sqlite database in memory.
    let (session_context, mut session, _, _) = seed_db().await;

    let created_session = session_context.create(session.clone()).await.unwrap();

    session.updated_at = created_session.updated_at;

    let fetched_session = session::Entity::find_by_id(created_session.id)
        .one(&session_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(session, created_session);
    assert_eq!(fetched_session, created_session);
}

#[tokio::test]
async fn create_default_created_at_test() {
    let t_min = Utc::now().timestamp();

    let (session_context, session, _, _) = seed_db().await;

    let _created_session = session_context.create(session.clone()).await.unwrap();

    let fetched_session = session::Entity::find_by_id(1)
        .one(&session_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    let t_max = Utc::now().timestamp();
    let t_actual = fetched_session.clone().updated_at.timestamp();

    assert!(t_min <= t_actual && t_actual <= t_max)
}

#[tokio::test]
async fn create_auto_increment_test() {
    // Setting up database and session context
    let (session_context, _, user, _) = seed_db().await;

    let sessions = create_sessions(2, user.id);

    // Creates the sessions in the database using the 'create' function
    let created_session1 = session_context.create(sessions[0].clone()).await.unwrap();
    let created_session2 = session_context.create(sessions[1].clone()).await.unwrap();

    let fetched_session1 = session::Entity::find_by_id(created_session1.id)
        .one(&session_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    let fetched_session2 = session::Entity::find_by_id(created_session2.id)
        .one(&session_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    // Assert if the new_session, created_session, and fetched_session are the same
    assert_ne!(fetched_session1.id, fetched_session2.id);
    assert_ne!(created_session1.id, created_session2.id);
    assert_eq!(created_session1.id, fetched_session1.id);
    assert_eq!(created_session2.id, fetched_session2.id);
}

#[tokio::test]
async fn create_non_unique_refresh_token_test() {
    let (session_context, _, _, user) = seed_db().await;

    let mut sessions = create_sessions(2, user.id);

    sessions[1].refresh_token = sessions[0].refresh_token.clone();

    let _created_session1 = session_context.create(sessions[0].clone()).await.unwrap();
    let created_session2 = session_context.create(sessions[1].clone()).await;

    assert!(matches!(
        created_session2.unwrap_err().sql_err(),
        Some(SqlErr::UniqueConstraintViolation(_))
    ));
}

#[tokio::test]
async fn get_by_id_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let fetched_session = session_context
        .get_by_id(session.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(session, fetched_session);
}

#[tokio::test]
async fn get_by_non_existing_id_test() {
    let (session_context, _, _, _) = seed_db().await;

    let fetched_session = session_context.get_by_id(1).await.unwrap();

    assert!(fetched_session.is_none());
}

#[tokio::test]
async fn get_all_test() {
    let (session_context, _, user, _) = seed_db().await;

    let new_sessions = create_sessions(3, user.id);

    session::Entity::insert_many(to_active_models!(new_sessions.clone()))
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(session_context.get_all().await.unwrap().len(), 3);

    let mut sorted: Vec<session::Model> = new_sessions.clone();
    sorted.sort_by_key(|k| k.id);

    for (i, session) in sorted.into_iter().enumerate() {
        assert_eq!(session, new_sessions[i]);
    }
}

#[tokio::test]
async fn get_all_empty_test() {
    let (session_context, _, _, _) = seed_db().await;

    let result = session_context.get_all().await.unwrap();
    let empty_accesses: Vec<session::Model> = vec![];

    assert_eq!(empty_accesses, result);
}

#[tokio::test]
async fn update_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    //A session has nothing to update
    let mut new_session = session::Model { ..session };

    let mut updated_session = session_context.update(new_session.clone()).await.unwrap();

    let fetched_session = session::Entity::find_by_id(updated_session.id)
        .one(&session_context.db_context.get_connection())
        .await
        .unwrap()
        .unwrap();

    new_session.updated_at = fetched_session.updated_at;
    updated_session.updated_at = fetched_session.updated_at;

    assert_eq!(new_session, updated_session);
    assert_eq!(updated_session, fetched_session);
}

#[tokio::test]
async fn update_does_not_modify_id_test() {
    let (session_context, session, _, _) = seed_db().await;
    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let updated_session = session::Model {
        id: &session.id + 1,
        ..session.clone()
    };
    let res = session_context.update(updated_session.clone()).await;

    assert!(matches!(res.unwrap_err(), DbErr::RecordNotUpdated));
}

#[tokio::test]
async fn update_does_modifies_updated_at_automatically_test() {
    let (session_context, mut session, _, _) = seed_db().await;
    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let updated_session = session::Model {
        updated_at: session.clone().updated_at.add(Duration::seconds(1)),
        ..session.clone()
    };
    let res = session_context
        .update(updated_session.clone())
        .await
        .unwrap();

    assert!(session.updated_at < res.updated_at);

    session.updated_at = res.updated_at;

    assert_eq!(session, res);
}

#[tokio::test]
async fn update_does_not_modify_user_id_test() {
    let (session_context, mut session, _, _) = seed_db().await;
    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let updated_session = session::Model {
        user_id: &session.user_id + 1,
        ..session.clone()
    };
    let res = session_context
        .update(updated_session.clone())
        .await
        .unwrap();

    session.updated_at = res.updated_at;

    assert_eq!(session, res);
}

#[tokio::test]
async fn update_non_existing_id_test() {
    let (session_context, session, _, _) = seed_db().await;

    let updated_session = session_context.update(session.clone()).await;

    assert!(matches!(
        updated_session.unwrap_err(),
        DbErr::RecordNotUpdated
    ));
}

#[tokio::test]
async fn delete_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let deleted_session = session_context.delete(session.id).await.unwrap();

    let all_sessions = session::Entity::find()
        .all(&session_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(session, deleted_session);
    assert!(all_sessions.is_empty());
}

#[tokio::test]
async fn delete_cascade_in_use_test() {
    let (session_context, session, _, model) = seed_db().await;

    let in_use = create_in_uses(1, model.id, session.id)[0].clone();

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();
    in_use::Entity::insert(in_use.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    session_context.delete(session.id).await.unwrap();

    let all_sessions = session::Entity::find()
        .all(&session_context.db_context.get_connection())
        .await
        .unwrap();
    let all_in_uses = in_use::Entity::find()
        .all(&session_context.db_context.get_connection())
        .await
        .unwrap();

    assert_eq!(all_sessions.len(), 0);
    assert_eq!(all_in_uses.len(), 0);
}

#[tokio::test]
async fn delete_non_existing_id_test() {
    let (session_context, _, _, _) = seed_db().await;

    let deleted_session = session_context.delete(1).await;

    assert!(matches!(
        deleted_session.unwrap_err(),
        DbErr::RecordNotFound(_)
    ));
}

#[tokio::test]
async fn get_by_token_refresh_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let fetched_session = session_context
        .get_by_token(TokenType::RefreshToken, session.refresh_token.clone())
        .await
        .unwrap();

    assert_eq!(
        fetched_session.unwrap().refresh_token,
        session.refresh_token
    );
}

#[tokio::test]
async fn get_by_token_access_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    let fetched_session = session_context
        .get_by_token(TokenType::AccessToken, session.access_token.clone())
        .await
        .unwrap();

    assert_eq!(fetched_session.unwrap().access_token, session.access_token);
}

#[tokio::test]
async fn delete_by_token_refresh_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    session_context
        .delete_by_token(TokenType::RefreshToken, session.refresh_token.clone())
        .await
        .unwrap();

    let fetched_session = session_context
        .get_by_token(TokenType::RefreshToken, session.refresh_token.clone())
        .await
        .unwrap();

    assert!(fetched_session.is_none());
}

#[tokio::test]
async fn delete_by_token_access_test() {
    let (session_context, session, _, _) = seed_db().await;

    session::Entity::insert(session.clone().into_active_model())
        .exec(&session_context.db_context.get_connection())
        .await
        .unwrap();

    session_context
        .delete_by_token(TokenType::AccessToken, session.access_token.clone())
        .await
        .unwrap();

    let fetched_session = session_context
        .get_by_token(TokenType::AccessToken, session.access_token.clone())
        .await
        .unwrap();

    assert!(fetched_session.is_none());
}
