use std::fmt::{Debug, Display, Formatter};

use cqrs_es::AggregateError;
use persist_es::PersistenceError;

#[derive(Debug, PartialEq)]
pub enum DynamoAggregateError {
    OptimisticLock,
    ConnectionError(String),
    UnknownError(String),
}

impl Display for DynamoAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamoAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            DynamoAggregateError::UnknownError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::ConnectionError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DynamoAggregateError {}

impl From<DynamoAggregateError> for AggregateError {
    fn from(err: DynamoAggregateError) -> Self {
        match err {
            DynamoAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            DynamoAggregateError::UnknownError(msg) => AggregateError::TechnicalError(msg),
            DynamoAggregateError::ConnectionError(msg) => AggregateError::TechnicalError(msg),
        }
    }
}

impl From<serde_json::Error> for DynamoAggregateError {
    fn from(err: serde_json::Error) -> Self {
        DynamoAggregateError::UnknownError(err.to_string())
    }
}

impl From<DynamoAggregateError> for PersistenceError {
    fn from(err: DynamoAggregateError) -> Self {
        match err {
            DynamoAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            DynamoAggregateError::UnknownError(msg) => PersistenceError::UnknownError(msg),
            DynamoAggregateError::ConnectionError(msg) => PersistenceError::ConnectionError(msg),
        }
    }
}
