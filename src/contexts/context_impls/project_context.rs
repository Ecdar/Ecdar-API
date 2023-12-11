use crate::contexts::context_traits::{
    DatabaseContextTrait, EntityContextTrait, ProjectContextTrait,
};
use crate::entities::{access, project, query};

use crate::api::server::protobuf::ProjectInfo;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, JoinType, ModelTrait,
    QueryFilter, QuerySelect, RelationTrait, Set, Unchanged,
};
use std::sync::Arc;

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
#[path = "../../tests/contexts/project_context.rs"]
mod project_context_tests;
