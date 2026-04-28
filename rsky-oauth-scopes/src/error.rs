use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScopeError {
    #[error("scope token must not be empty")]
    Empty,
    #[error("scope token must not contain whitespace: {0:?}")]
    ContainsWhitespace(String),
    #[error("invalid scope: {0}")]
    Invalid(String),
}
