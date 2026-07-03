use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("failed to acquire registry read lock: {0}")]
    ReadLockPoisoned(String),

    #[error("failed to acquire registry write lock: {0}")]
    WriteLockPoisoned(String),

    #[error("registry add failed: {0}")]
    KeyAlreadyExists(String),

    #[error("registry remove failed: {0}")]
    KeyNotFound(String),
}
