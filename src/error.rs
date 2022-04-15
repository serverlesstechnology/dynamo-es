use std::fmt::{Debug, Display, Formatter};

use aws_sdk_dynamodb::error::TransactWriteItemsError;
use aws_sdk_dynamodb::error::{QueryError, TransactWriteItemsErrorKind};
use aws_sdk_dynamodb::types::SdkError;
use cqrs_es::persist::PersistenceError;
use cqrs_es::AggregateError;

#[derive(Debug)]
pub enum DynamoAggregateError {
    OptimisticLock,
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    TransactionListTooLong(usize),
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for DynamoAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamoAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            DynamoAggregateError::ConnectionError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::DeserializationError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::UnknownError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::TransactionListTooLong(length) => write!(f, "Too many operations: {}, DynamoDb supports only up to 25 operations per transactions", length),
        }
    }
}

impl std::error::Error for DynamoAggregateError {}

impl<T: std::error::Error> From<DynamoAggregateError> for AggregateError<T> {
    fn from(error: DynamoAggregateError) -> Self {
        match error {
            DynamoAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            DynamoAggregateError::ConnectionError(err) => {
                AggregateError::DatabaseConnectionError(err)
            }
            DynamoAggregateError::DeserializationError(err) => {
                AggregateError::DeserializationError(err)
            }
            DynamoAggregateError::TransactionListTooLong(_) => {
                AggregateError::UnexpectedError(Box::new(error))
            }
            DynamoAggregateError::UnknownError(err) => AggregateError::UnexpectedError(err),
        }
    }
}

impl From<serde_json::Error> for DynamoAggregateError {
    fn from(err: serde_json::Error) -> Self {
        DynamoAggregateError::UnknownError(Box::new(err))
    }
}

impl From<SdkError<TransactWriteItemsError>> for DynamoAggregateError {
    fn from(error: SdkError<TransactWriteItemsError>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => DynamoAggregateError::UnknownError(err),
            SdkError::TimeoutError(err) => DynamoAggregateError::UnknownError(err),
            SdkError::DispatchFailure(err) => DynamoAggregateError::UnknownError(Box::new(err)),
            SdkError::ResponseError { err, .. } => DynamoAggregateError::UnknownError(err),
            SdkError::ServiceError { err, .. } => match &err.kind {
                TransactWriteItemsErrorKind::TransactionCanceledException(cancellation) => {
                    if let Some(reasons) = &cancellation.cancellation_reasons {
                        for reason in reasons {
                            if let Some(code) = &reason.code {
                                if code.as_str() == "ConditionalCheckFailed" {
                                    return DynamoAggregateError::OptimisticLock;
                                }
                            }
                        }
                    }
                    DynamoAggregateError::UnknownError(Box::new(err))
                }
                _ => DynamoAggregateError::UnknownError(Box::new(err)),
            },
        }
    }
}

impl From<SdkError<QueryError>> for DynamoAggregateError {
    fn from(error: SdkError<QueryError>) -> Self {
        match error {
            SdkError::ConstructionFailure(err) => DynamoAggregateError::UnknownError(err),
            SdkError::TimeoutError(err) => DynamoAggregateError::UnknownError(err),
            SdkError::DispatchFailure(err) => DynamoAggregateError::UnknownError(Box::new(err)),
            SdkError::ResponseError { err, .. } => DynamoAggregateError::UnknownError(err),
            SdkError::ServiceError { err, .. } => DynamoAggregateError::UnknownError(Box::new(err)),
        }
    }
}

impl From<DynamoAggregateError> for PersistenceError {
    fn from(error: DynamoAggregateError) -> Self {
        match error {
            DynamoAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            DynamoAggregateError::ConnectionError(err) => PersistenceError::ConnectionError(err),
            DynamoAggregateError::DeserializationError(err) => {
                PersistenceError::DeserializationError(err)
            }
            DynamoAggregateError::TransactionListTooLong(_) => {
                PersistenceError::UnknownError(Box::new(error))
            }
            DynamoAggregateError::UnknownError(err) => PersistenceError::UnknownError(err),
        }
    }
}
