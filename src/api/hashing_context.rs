use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};

pub trait HashingContextTrait: Send + Sync {
    fn hash_password(&self, password: String) -> Result<String, BcryptError>;
    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError>;
}

pub struct HashingContext;

impl HashingContextTrait for HashingContext {
    fn hash_password(&self, password: String) -> Result<String, BcryptError> {
        hash(password, DEFAULT_COST)
        // .expect("failed to hash password")
    }

    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError> {
        verify(password, hash)
        // .expect("failed to verify password")
    }
}
