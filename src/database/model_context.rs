use crate::database::database_context::DatabaseContextTrait;
use crate::entities::model::{ActiveModel, Model};
use crate::entities::prelude::Model as ModelEntity;
use crate::entities::query;
use crate::EntityContextTrait;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, RuntimeErr, Set, Unchanged,
};

pub struct ModelContext {
    db_context: Box<dyn DatabaseContextTrait>,
}

pub trait ModelContextTrait: EntityContextTrait<Model> {}

impl ModelContextTrait for ModelContext {}

#[async_trait]
impl EntityContextTrait<Model> for ModelContext {
    fn new(db_context: Box<dyn DatabaseContextTrait>) -> ModelContext {
        ModelContext { db_context }
    }

    /// Used for creating a Model entity
    async fn create(&self, entity: Model) -> Result<Model, DbErr> {
        let model = ActiveModel {
            id: Default::default(),
            name: Set(entity.name),
            components_info: Set(entity.components_info),
            owner_id: Set(entity.owner_id),
        };
        let model: Model = model.insert(&self.db_context.get_connection()).await?;
        Ok(model)
    }

    async fn get_by_id(&self, entity_id: i32) -> Result<Option<Model>, DbErr> {
        ModelEntity::find_by_id(entity_id)
            .one(&self.db_context.get_connection())
            .await
    }

    async fn get_all(&self) -> Result<Vec<Model>, DbErr> {
        ModelEntity::find()
            .all(&self.db_context.get_connection())
            .await
    }

    async fn update(&self, entity: Model) -> Result<Model, DbErr> {
        let existing_model = self.get_by_id(entity.id).await?;

        return match existing_model {
            None => Err(DbErr::RecordNotUpdated),
            Some(existing_model) => {
                let queries: Vec<query::Model> = existing_model
                    .find_related(query::Entity)
                    .all(&self.db_context.get_connection())
                    .await?;
                for q in queries.iter() {
                    let mut aq = q.clone().into_active_model();
                    aq.outdated = Set(true);
                    aq.update(&self.db_context.get_connection()).await?;
                }
                ActiveModel {
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

    async fn delete(&self, entity_id: i32) -> Result<Model, DbErr> {
        let model = self.get_by_id(entity_id).await?;
        match model {
            None => Err(DbErr::Exec(RuntimeErr::Internal(
                "No record was deleted".into(),
            ))),
            Some(model) => {
                ModelEntity::delete_by_id(entity_id)
                    .exec(&self.db_context.get_connection())
                    .await?;
                Ok(model)
            }
        }
    }
}

//#[cfg(test)]
//#[path = "../tests/database/model_context.rs"]
//mod tests;
