use crate::services::service_traits::hashing_service_trait::HashingServiceTrait;
use bcrypt::{hash, verify, DEFAULT_COST, BcryptError};

pub struct HashingService;

impl HashingServiceTrait for HashingService {
    fn hash_password(&self, password: String) -> Result<String, BcryptError> {
        hash(password, DEFAULT_COST)
    }

    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError> {
        verify(password, hash)
    }
}
