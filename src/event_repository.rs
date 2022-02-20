use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, Put, TransactWriteItem};
use aws_sdk_dynamodb::output::QueryOutput;
use aws_sdk_dynamodb::{Blob, Client};
use cqrs_es::Aggregate;
use persist_es::{PersistedEventRepository, PersistenceError, SerializedEvent, SerializedSnapshot};
use serde_json::Value;

use crate::error::DynamoAggregateError;
use crate::helpers::{att_as_number, att_as_string, att_as_value, commit_transactions};

/// A snapshot backed event repository for use in backing a `PersistedSnapshotStore`.
pub struct DynamoEventRepository {
    client: aws_sdk_dynamodb::client::Client,
}

const EVENT_TABLE: &str = "Events";
const SNAPSHOT_TABLE: &str = "Snapshots";

impl DynamoEventRepository {
    /// Creates a new `DynamoEventRepository` from the provided database connection.
    ///
    /// ```ignore
    /// let store = DynamoEventRepository::<MyAggregate>::new(client);
    /// ```
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub(crate) async fn insert_events<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
    ) -> Result<(), DynamoAggregateError> {
        let (transactions, _) = Self::build_event_put_transactions(events);
        commit_transactions(&self.client, transactions).await?;
        Ok(())
    }

    fn build_event_put_transactions(events: &[SerializedEvent]) -> (Vec<TransactWriteItem>, usize) {
        let mut current_sequence: usize = 0;
        let mut transactions: Vec<TransactWriteItem> = Vec::default();
        for event in events {
            current_sequence = event.sequence;
            let aggregate_type_and_id =
                AttributeValue::S(format!("{}:{}", &event.aggregate_type, &event.aggregate_id));
            let aggregate_type = AttributeValue::S(String::from(&event.aggregate_type));
            let aggregate_id = AttributeValue::S(String::from(&event.aggregate_id));
            let sequence = AttributeValue::N(String::from(&event.sequence.to_string()));
            let event_version = AttributeValue::S(String::from(&event.event_version));
            let event_type = AttributeValue::S(String::from(&event.event_type));
            let payload_blob = serde_json::to_vec(&event.payload).unwrap();
            let payload = AttributeValue::B(Blob::new(payload_blob));
            let metadata_blob = serde_json::to_vec(&event.metadata).unwrap();
            let metadata = AttributeValue::B(Blob::new(metadata_blob));

            let put = Put::builder()
                .table_name(EVENT_TABLE)
                .item("AggregateTypeAndId", aggregate_type_and_id)
                .item("AggregateIdSequence", sequence)
                .item("AggregateType", aggregate_type)
                .item("AggregateId", aggregate_id)
                .item("EventVersion", event_version)
                .item("EventType", event_type)
                .item("Payload", payload)
                .item("Metadata", metadata)
                .condition_expression("attribute_not_exists( AggregateIdSequence )")
                .build();
            let write_item = TransactWriteItem::builder().put(put).build();
            transactions.push(write_item);
        }
        (transactions, current_sequence)
    }

    async fn query_events(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, DynamoAggregateError> {
        let query_output = self
            .query_table(aggregate_type, aggregate_id, EVENT_TABLE)
            .await?;
        let mut result: Vec<SerializedEvent> = Default::default();
        if let Some(entries) = query_output.items {
            for entry in entries {
                result.push(serialized_event(entry));
            }
        }
        Ok(result)
    }

    pub(crate) async fn update_snapshot<A: Aggregate>(
        &self,
        aggregate_payload: Value,
        aggregate_id: String,
        current_snapshot: usize,
        events: &[SerializedEvent],
    ) -> Result<(), DynamoAggregateError> {
        let expected_snapshot = current_snapshot - 1;
        let (mut transactions, current_sequence) = Self::build_event_put_transactions(events);
        let aggregate_type_and_id =
            AttributeValue::S(format!("{}:{}", A::aggregate_type(), &aggregate_id));
        let aggregate_type = AttributeValue::S(A::aggregate_type().to_string());
        let aggregate_id = AttributeValue::S(aggregate_id);
        let current_sequence = AttributeValue::N(current_sequence.to_string());
        let current_snapshot = AttributeValue::N(current_snapshot.to_string());
        let payload_blob = serde_json::to_vec(&aggregate_payload).unwrap();
        let payload = AttributeValue::B(Blob::new(payload_blob));
        let expected_snapshot = AttributeValue::N(expected_snapshot.to_string());
        transactions.push(TransactWriteItem::builder()
            .put(Put::builder()
                .table_name(SNAPSHOT_TABLE)
                .item("AggregateTypeAndId", aggregate_type_and_id)
                .item("AggregateType", aggregate_type)
                .item("AggregateId", aggregate_id)
                .item("CurrentSequence", current_sequence)
                .item("CurrentSnapshot", current_snapshot)
                .item("Payload", payload)
                .condition_expression("attribute_not_exists(CurrentSnapshot) OR (CurrentSnapshot  = :current_snapshot)")
                .expression_attribute_values(":current_snapshot", expected_snapshot)
                .build())
            .build());
        commit_transactions(&self.client, transactions).await?;
        Ok(())
    }

    async fn query_table(
        &self,
        aggregate_type: &str,
        aggregate_id: &str,
        table: &str,
    ) -> Result<QueryOutput, DynamoAggregateError> {
        Ok(self
            .client
            .query()
            .table_name(table)
            .key_condition_expression("#agg_type_id = :agg_type_id")
            .expression_attribute_names("#agg_type_id", "AggregateTypeAndId")
            .expression_attribute_values(
                ":agg_type_id",
                AttributeValue::S(format!("{}:{}", aggregate_type, aggregate_id)),
            )
            .send()
            .await?)
    }
}

fn serialized_event(entry: HashMap<String, AttributeValue>) -> SerializedEvent {
    let aggregate_id = att_as_string(entry.get("AggregateId"));
    let sequence = att_as_number(entry.get("AggregateIdSequence"));
    let aggregate_type = att_as_string(entry.get("AggregateType"));
    let event_type = att_as_string(entry.get("EventType"));
    let event_version = att_as_string(entry.get("EventVersion"));
    let payload = att_as_value(entry.get("Payload"));
    let metadata = att_as_value(entry.get("Metadata"));
    SerializedEvent {
        aggregate_id,
        sequence,
        aggregate_type,
        event_type,
        event_version,
        payload,
        metadata,
    }
}

#[async_trait]
impl PersistedEventRepository for DynamoEventRepository {
    async fn get_events<A: Aggregate>(
        &self,
        aggregate_id: &str,
    ) -> Result<Vec<SerializedEvent>, PersistenceError> {
        let request = self.query_events(A::aggregate_type(), aggregate_id).await?;
        Ok(request)
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
        aggregate_id: &str,
    ) -> Result<Option<SerializedSnapshot>, PersistenceError> {
        let query_output = self
            .query_table(A::aggregate_type(), aggregate_id, SNAPSHOT_TABLE)
            .await?;
        let query_items_vec = match query_output.items {
            None => return Ok(None),
            Some(items) => items,
        };
        if query_items_vec.is_empty() {
            return Ok(None);
        }
        let query_item = query_items_vec.get(0).unwrap();
        let aggregate = att_as_value(query_item.get("Payload"));
        let current_sequence = att_as_number(query_item.get("CurrentSequence"));
        let current_snapshot = att_as_number(query_item.get("CurrentSnapshot"));

        Ok(Some(SerializedSnapshot {
            aggregate_id: aggregate_id.to_string(),
            aggregate,
            current_sequence,
            current_snapshot,
        }))
    }

    async fn persist<A: Aggregate>(
        &self,
        events: &[SerializedEvent],
        snapshot_update: Option<(String, Value, usize)>,
    ) -> Result<(), PersistenceError> {
        match snapshot_update {
            None => {
                self.insert_events::<A>(events).await?;
            }
            Some((aggregate_id, aggregate, current_snapshot)) => {
                self.update_snapshot::<A>(aggregate, aggregate_id, current_snapshot, events)
                    .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use cqrs_es::EventStore;
    use persist_es::PersistedEventRepository;

    use crate::error::DynamoAggregateError;
    use crate::testing::tests::{
        new_test_event_store, new_test_metadata, new_test_snapshot_store, snapshot_context,
        test_dynamodb_client, test_event_envelope, Created, SomethingElse, TestAggregate,
        TestEvent, Tested,
    };
    use crate::DynamoEventRepository;

    #[tokio::test]
    async fn commit_and_load_events() {
        let client = test_dynamodb_client().await;
        let event_store = new_test_event_store(client).await;
        let id = uuid::Uuid::new_v4().to_string();
        assert_eq!(0, event_store.load(id.as_str()).await.len());
        let context = event_store.load_aggregate(id.as_str()).await;

        event_store
            .commit(
                vec![
                    TestEvent::Created(Created {
                        id: "test_event_A".to_string(),
                    }),
                    TestEvent::Tested(Tested {
                        test_name: "test A".to_string(),
                    }),
                ],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();

        assert_eq!(2, event_store.load(id.as_str()).await.len());
        let context = event_store.load_aggregate(id.as_str()).await;

        event_store
            .commit(
                vec![TestEvent::Tested(Tested {
                    test_name: "test B".to_string(),
                })],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();
        assert_eq!(3, event_store.load(id.as_str()).await.len());
    }

    #[tokio::test]
    async fn commit_and_load_events_snapshot_store() {
        let client = test_dynamodb_client().await;
        let event_store = new_test_snapshot_store(client).await;
        let id = uuid::Uuid::new_v4().to_string();
        assert_eq!(0, event_store.load(id.as_str()).await.len());
        let context = event_store.load_aggregate(id.as_str()).await;

        event_store
            .commit(
                vec![
                    TestEvent::Created(Created {
                        id: "test_event_A".to_string(),
                    }),
                    TestEvent::Tested(Tested {
                        test_name: "test A".to_string(),
                    }),
                ],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();

        assert_eq!(2, event_store.load(id.as_str()).await.len());
        let context = event_store.load_aggregate(id.as_str()).await;

        event_store
            .commit(
                vec![TestEvent::Tested(Tested {
                    test_name: "test B".to_string(),
                })],
                context,
                new_test_metadata(),
            )
            .await
            .unwrap();
        assert_eq!(3, event_store.load(id.as_str()).await.len());
    }

    #[tokio::test]
    async fn event_repositories() {
        let client = test_dynamodb_client().await;
        let id = uuid::Uuid::new_v4().to_string();
        let event_repo = DynamoEventRepository::new(client.clone());
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert!(events.is_empty());

        event_repo
            .insert_events::<TestAggregate>(&[
                test_event_envelope(&id, 1, TestEvent::Created(Created { id: id.clone() })),
                test_event_envelope(
                    &id,
                    2,
                    TestEvent::Tested(Tested {
                        test_name: "a test was run".to_string(),
                    }),
                ),
            ])
            .await
            .unwrap();
        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());
        events.iter().for_each(|e| assert_eq!(&id, &e.aggregate_id));

        // Optimistic lock error
        let result = event_repo
            .insert_events::<TestAggregate>(&[
                test_event_envelope(
                    &id,
                    3,
                    TestEvent::SomethingElse(SomethingElse {
                        description: "this should not persist".to_string(),
                    }),
                ),
                test_event_envelope(
                    &id,
                    2,
                    TestEvent::SomethingElse(SomethingElse {
                        description: "bad sequence number".to_string(),
                    }),
                ),
            ])
            .await
            .unwrap_err();
        match result {
            DynamoAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
        assert_eq!(2, events.len());
    }

    #[tokio::test]
    async fn snapshot_repositories() {
        let client = test_dynamodb_client().await;
        let id = uuid::Uuid::new_v4().to_string();
        let repo = DynamoEventRepository::new(client.clone());
        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(None, snapshot);

        let test_description = "some test snapshot here".to_string();
        let test_tests = vec!["testA".to_string(), "testB".to_string()];
        repo.update_snapshot::<TestAggregate>(
            serde_json::to_value(TestAggregate {
                id: id.clone(),
                description: test_description.clone(),
                tests: test_tests.clone(),
            })
            .unwrap(),
            id.clone(),
            1,
            &vec![],
        )
        .await
        .unwrap();

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                1,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: test_description.clone(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );

        // sequence iterated, does update
        repo.update_snapshot::<TestAggregate>(
            serde_json::to_value(TestAggregate {
                id: id.clone(),
                description: "a test description that should be saved".to_string(),
                tests: test_tests.clone(),
            })
            .unwrap(),
            id.clone(),
            2,
            &vec![],
        )
        .await
        .unwrap();

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                2,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );

        // sequence out of order or not iterated, does not update
        let result = repo
            .update_snapshot::<TestAggregate>(
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should not be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
                id.clone(),
                2,
                &vec![],
            )
            .await
            .unwrap_err();
        match result {
            DynamoAggregateError::OptimisticLock => {}
            _ => panic!("invalid error result found during insert: {}", result),
        };

        let snapshot = repo.get_snapshot::<TestAggregate>(&id).await.unwrap();
        assert_eq!(
            Some(snapshot_context(
                id.clone(),
                0,
                2,
                serde_json::to_value(TestAggregate {
                    id: id.clone(),
                    description: "a test description that should be saved".to_string(),
                    tests: test_tests.clone(),
                })
                .unwrap(),
            )),
            snapshot
        );
    }
}
