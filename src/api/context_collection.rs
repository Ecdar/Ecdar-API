use crate::api::hashing_context::HashingContextTrait;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::database::context_traits::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ContextCollection {
    pub(crate) access_context: Arc<dyn AccessContextTrait>,
    pub(crate) in_use_context: Arc<dyn InUseContextTrait>,
    pub(crate) project_context: Arc<dyn ProjectContextTrait>,
    pub(crate) query_context: Arc<dyn QueryContextTrait>,
    pub(crate) session_context: Arc<dyn SessionContextTrait>,
    pub(crate) user_context: Arc<dyn UserContextTrait>,
    pub(crate) reveaal_context: Arc<dyn EcdarBackend>,
    pub(crate) hashing_context: Arc<dyn HashingContextTrait>,
}
