use bcrypt::{hash, verify, DEFAULT_COST};

pub trait HashingContextTrait: Send + Sync {
    fn hash_password(&self, password: String) -> String;
    fn verify_password(&self, password: String, hash: &str) -> bool;
}

pub struct HashingContext;

impl HashingContextTrait for HashingContext {
    fn hash_password(&self, password: String) -> String {
        hash(password, DEFAULT_COST).unwrap()
    }

    fn verify_password(&self, password: String, hash: &str) -> bool {
        verify(password, hash).unwrap()
    }
}
