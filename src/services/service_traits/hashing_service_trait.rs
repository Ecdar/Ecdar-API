pub trait HashingServiceTrait: Send + Sync {
    fn hash_password(&self, password: String) -> String;
    fn verify_password(&self, password: String, hash: &str) -> bool;
}
