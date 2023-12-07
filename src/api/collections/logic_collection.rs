use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::logics::logic_traits::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct LogicCollection {
    pub(crate) access_logic: Arc<dyn AccessLogicTrait>,
    pub(crate) project_logic: Arc<dyn ProjectLogicTrait>,
    pub(crate) query_logic: Arc<dyn QueryLogicTrait>,
    pub(crate) session_logic: Arc<dyn SessionLogicTrait>,
    pub(crate) user_logic: Arc<dyn UserLogicTrait>,
    pub(crate) reveaal_logic: Arc<dyn EcdarBackend>,
}
