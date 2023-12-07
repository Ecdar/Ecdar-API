use crate::api::logic_traits::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct LogicCollection {
    pub(crate) access_logic: Arc<dyn AccessLogicTrait>,
    pub(crate) project_logic: Arc<dyn ProjectLogicTrait>,
    pub(crate) query_logic: Arc<dyn QueryLogicTrait>,
    pub(crate) session_logic: Arc<dyn SessionLogicTrait>,
    pub(crate) user_logic: Arc<dyn UserLogicTrait>,
}
