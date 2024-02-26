use crate::services::{HashingService, ReveaalServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceCollection {
    pub(crate) hashing_service: Arc<dyn HashingService>,
    pub(crate) reveaal_service: Arc<dyn ReveaalServiceTrait>,
}
