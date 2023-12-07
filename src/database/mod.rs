//! # Description
//! The purpose of this module is to separate database communication details from the API endpoints.
//! This allows the changing of database backends (currently `Postgres` and `SQLite`).
//! This is done by implementing [`database_context::DatabaseContextTrait`] and [`entity_context::EntityContextTrait`]
pub mod access_context;
pub mod database_context;
pub mod entity_context;
pub mod in_use_context;
pub mod model_context;
pub mod query_context;
pub mod session_context;
pub mod user_context;
