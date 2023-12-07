use crate::api::logic_traits::HashingContextTrait;
use bcrypt::{hash, verify, DEFAULT_COST};

pub struct HashingContext;

impl HashingContextTrait for HashingContext {
    fn hash_password(&self, password: String) -> String {
        hash(password, DEFAULT_COST).unwrap()
    }

    fn verify_password(&self, password: String, hash: &str) -> bool {
        verify(password, hash).unwrap()
    }
}
