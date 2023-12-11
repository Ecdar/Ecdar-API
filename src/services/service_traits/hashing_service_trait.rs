use bcrypt::BcryptError;

pub trait HashingServiceTrait: Send + Sync {
    fn hash_password(&self, password: String) -> Result<String, BcryptError>;
    fn verify_password(&self, password: String, hash: &str) -> Result<bool, BcryptError>;
}
