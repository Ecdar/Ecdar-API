use crate::api::hashing_context::HashingContextTrait;
use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::database::access_context::AccessContextTrait;
use crate::database::in_use_context::InUseContextTrait;
use crate::database::model_context::ModelContextTrait;
use crate::database::query_context::QueryContextTrait;
use crate::database::session_context::SessionContextTrait;
use crate::database::user_context::UserContextTrait;
use std::sync::Arc;

#[derive(Clone)]
pub struct ContextCollection {
    pub(crate) access_context: Arc<dyn AccessContextTrait>,
    pub(crate) in_use_context: Arc<dyn InUseContextTrait>,
    pub(crate) model_context: Arc<dyn ModelContextTrait>,
    pub(crate) query_context: Arc<dyn QueryContextTrait>,
    pub(crate) session_context: Arc<dyn SessionContextTrait>,
    pub(crate) user_context: Arc<dyn UserContextTrait>,
    pub(crate) reveaal_context: Arc<dyn EcdarBackend>,
    pub(crate) hashing_context: Arc<dyn HashingContextTrait>,
}
