use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("no party found")]
    NoPartyFound,
}
