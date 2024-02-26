use crate::services::{HashingServiceTrait, ReveaalServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct ServiceCollection {
    pub(crate) hashing_service: Arc<dyn HashingServiceTrait>,
    pub(crate) reveaal_service: Arc<dyn ReveaalServiceTrait>,
}
