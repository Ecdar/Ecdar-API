use crate::contexts::context_traits::EntityContextTrait;
use crate::entities::in_use;

pub trait InUseContextTrait: EntityContextTrait<in_use::Model> {}
