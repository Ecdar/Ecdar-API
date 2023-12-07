use crate::services::service_traits::hashing_service_trait::HashingServiceTrait;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct HashingService;

impl HashingServiceTrait for HashingService {
    fn hash_password(&self, password: String) -> String {
        hash(password, DEFAULT_COST).unwrap()
    }

    fn verify_password(&self, password: String, hash: &str) -> bool {
        verify(password, hash).unwrap()
    }
}
