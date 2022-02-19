use aws_sdk_dynamodb::error::TransactWriteItemsError;
use aws_sdk_dynamodb::error::{QueryError, ScanError, TransactWriteItemsErrorKind};
use aws_sdk_dynamodb::SdkError;
use std::fmt::{Debug, Display, Formatter};

use cqrs_es::AggregateError;
use persist_es::PersistenceError;

#[derive(Debug)]
pub enum DynamoAggregateError {
    OptimisticLock,
    ConnectionError(Box<dyn std::error::Error + Send + Sync + 'static>),
    DeserializationError(Box<dyn std::error::Error + Send + Sync + 'static>),
    UnknownError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl Display for DynamoAggregateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamoAggregateError::OptimisticLock => write!(f, "optimistic lock error"),
            DynamoAggregateError::ConnectionError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::DeserializationError(msg) => write!(f, "{}", msg),
            DynamoAggregateError::UnknownError(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for DynamoAggregateError {}

impl<T: std::error::Error> From<DynamoAggregateError> for AggregateError<T> {
    fn from(err: DynamoAggregateError) -> Self {
        match err {
            DynamoAggregateError::OptimisticLock => AggregateError::AggregateConflict,
            DynamoAggregateError::ConnectionError(err) => {
                AggregateError::DatabaseConnectionError(err)
            }
            DynamoAggregateError::DeserializationError(err) => {
                AggregateError::DeserializationError(err)
            }
            DynamoAggregateError::UnknownError(err) => AggregateError::TechnicalError(err),
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
            SdkError::ResponseError { err, raw } => DynamoAggregateError::UnknownError(err),
            SdkError::ServiceError { err, raw } => match &err.kind {
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
            SdkError::ResponseError { err, raw } => DynamoAggregateError::UnknownError(err),
            SdkError::ServiceError { err, raw } => {
                DynamoAggregateError::UnknownError(Box::new(err))
            }
        }
    }
}

impl From<DynamoAggregateError> for PersistenceError {
    fn from(err: DynamoAggregateError) -> Self {
        match err {
            DynamoAggregateError::OptimisticLock => PersistenceError::OptimisticLockError,
            DynamoAggregateError::ConnectionError(err) => PersistenceError::ConnectionError(err),
            DynamoAggregateError::DeserializationError(err) => {
                PersistenceError::DeserializationError(err)
            }
            DynamoAggregateError::UnknownError(err) => PersistenceError::UnknownError(err),
        }
    }
}
