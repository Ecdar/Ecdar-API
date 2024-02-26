use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};

pub trait HashingServiceTrait: Send + Sync {
    fn hash_password(&self, password: String) -> Result<String, BcryptError>;
    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError>;
}

pub struct HashingService;

impl HashingServiceTrait for HashingService {
    fn hash_password(&self, password: String) -> Result<String, BcryptError> {
        hash(password, DEFAULT_COST)
    }

    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError> {
        verify(password, hash)
    }
}
