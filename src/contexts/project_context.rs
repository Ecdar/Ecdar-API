use crate::entities::{access, project, query};

use crate::api::server::protobuf::ProjectInfo;
use crate::contexts::{DatabaseContextTrait, EntityContextTrait};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, JoinType, ModelTrait,
    QueryFilter, QuerySelect, RelationTrait, Set, Unchanged,
};
use std::sync::Arc;

#[async_trait]
pub trait ProjectContextTrait: EntityContextTrait<project::Model> {
    /// Returns the projects owned by a given user id
    /// # Errors
    /// Errors on failed connection, execution error or constraint violations.
    async fn get_project_info_by_uid(&self, uid: i32) -> Result<Vec<ProjectInfo>, DbErr>;
}

pub struct ProjectContext {
    db_context: Arc<dyn DatabaseContextTrait>,
}

#[async_trait]
impl ProjectContextTrait for ProjectContext {
    async fn get_project_info_by_uid(&self, uid: i32) -> Result<Vec<ProjectInfo>, DbErr> {
        //join project, access and role tables
        access::Entity::find()
            .select_only()
            .column_as(project::Column::Id, "project_id")
            .column_as(project::Column::Name, "project_name")
            .column_as(project::Column::OwnerId, "project_owner_id")
            .column_as(access::Column::Role, "user_role_on_project")
            .join(JoinType::InnerJoin, access::Relation::Project.def())
            .join(JoinType::InnerJoin, access::Relation::Role.def())
            .group_by(project::Column::Id)
            .group_by(access::Column::Role)
            .filter(access::Column::UserId.eq(uid))
            .into_model::<ProjectInfo>()
            .all(&self.db_context.get_connection())
            .await
    }
}

impl ProjectContext {
    pub fn new(db_context: Arc<dyn DatabaseContextTrait>) -> ProjectContext {
        ProjectContext { db_context }
    }
}

#[async_trait]
impl EntityContextTrait<project::Model> for ProjectContext {
    /// Used for creating a project::Model entity
    /// # Example
    /// ```
    /// let project = project::Model {
    ///     id: Default::default(),
    ///     name: "project::Model name".to_owned(),
    ///     components_info: "{}".to_owned().parse().unwrap(),
    ///     owner_id: 1
    /// };
    /// let project_context: ProjectContext = ProjectContext::new(...);
    /// project_context.create(project);
    /// ```
    async fn create(&self, entity: project::Model) -> Result<project::Model, DbErr> {
        let project = project::ActiveModel {
            id: Default::default(),
            name: Set(entity.name),
            components_info: Set(entity.components_info),
            owner_id: Set(entity.owner_id),
        };
        let project: project::Model = project.insert(&self.db_context.get_connection()).await?;
        Ok(project)
    }

    /// Returns a single project entity (Uses primary key)
    /// # Example
    /// ```
    /// let project_context: ProjectContext = ProjectContext::new(...);
    /// let project = project_context.get_by_id(1).unwrap();
    /// ```
    async fn get_by_id(&self, entity_id: i32) -> Result<Option<project::Model>, DbErr> {
        project::Entity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    /// Returns a all project entities (Uses primary key)
    /// # Example
    /// ```
    /// let project_context: ProjectContext = ProjectContext::new(...);
    /// let project = project_context.get_all().unwrap();
    /// ```
    async fn get_all(&self) -> Result<Vec<project::Model>, DbErr> {
        project::Entity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    /// Updates a single project entity
    /// # Example
    /// ```
    /// let update_project = project::Model {
    ///     name: "new name",
    ///     ..original_project
    /// };
    ///
    /// let project_context: ProjectContext = ProjectContext::new(...);
    /// let project = project_context.update(update_project).unwrap();
    /// ```
    async fn update(&self, entity: project::Model) -> Result<project::Model, DbErr> {
        let existing_project = self.get_by_id(entity.id).await?;

        return match existing_project {
            None => Err(DbErr::RecordNotUpdated),
            Some(existing_project) => {
                let queries: Vec<query::Model> = existing_project
                    .find_related(query::Entity)
                    .all(&self.db_context.get_connection())
                    .await?;
                for q in queries.iter() {
                    let mut aq = q.clone().into_active_model();
                    aq.outdated = Set(true);
                    aq.update(&self.db_context.get_connection()).await?;
                }
                project::ActiveModel {
                    id: Unchanged(entity.id),
                    name: Set(entity.name),
                    components_info: Set(entity.components_info),
                    owner_id: Unchanged(entity.id),
                }
                .update(&self.db_context.get_connection())
                .await
            }
        };
    }

    /// Returns and deletes a single project entity
    /// # Example
    /// ```
    /// let project_context: ProjectContext = ProjectContext::new(...);
    /// let project = project_context.delete().unwrap();
    /// ```
    async fn delete(&self, entity_id: i32) -> Result<project::Model, DbErr> {
        let project = self.get_by_id(entity_id).await?;
        match project {
            None => Err(DbErr::RecordNotFound("No record was deleted".into())),
            Some(project) => {
                project::Entity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(project)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::helpers::*;
    use crate::{
        contexts::EntityContextTrait,
        contexts::ProjectContext,
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
}
