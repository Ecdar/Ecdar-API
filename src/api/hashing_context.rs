use bcrypt::{hash, verify, DEFAULT_COST};

pub trait HashingContextTrait: Send + Sync {
    fn hash_password(&self, password: String) -> String;
    fn verify_password(&self, password: String, hash: &str) -> bool;
}

pub struct HashingContext;

impl HashingContextTrait for HashingContext {
    //! Methods should not panic, but propogate their result to the caller
    fn hash_password(&self, password: String) -> String {
        hash(password, DEFAULT_COST).expect("failed to hash password")
    }

    fn verify_password(&self, password: String, hash: &str) -> bool {
        verify(password, hash).expect("failed to verify password")
    }
}
