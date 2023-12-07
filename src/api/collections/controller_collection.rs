use crate::api::server::server::ecdar_backend_server::EcdarBackend;
use crate::controllers::controller_traits::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct ControllerCollection {
    pub(crate) access_controller: Arc<dyn AccessControllerTrait>,
    pub(crate) project_controller: Arc<dyn ProjectControllerTrait>,
    pub(crate) query_controller: Arc<dyn QueryControllerTrait>,
    pub(crate) session_controller: Arc<dyn SessionControllerTrait>,
    pub(crate) user_controller: Arc<dyn UserControllerTrait>,
    pub(crate) reveaal_controller: Arc<dyn EcdarBackend>,
}
