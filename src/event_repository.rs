use async_trait::async_trait;
use cqrs_es::Aggregate;
use persist_es::{PersistedEventRepository, PersistenceError, SerializedEvent, SerializedSnapshot};
use serde_json::Value;

/// A snapshot backed event repository for use in backing a `PersistedSnapshotStore`.
pub struct DynamoEventRepository {}

#[async_trait]
impl PersistedEventRepository for DynamoEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_last_events<A: Aggregate>(
        &self,
        _aggregate_id: &str,
        _number_events: usize,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        todo!()
    }

    async fn get_snapshot<A: Aggregate>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        todo!()
    }

    async fn persist<A: Aggregate>(
        &self,
        _events: &[SerializedEvent],
        _snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        todo!()
    }
}
