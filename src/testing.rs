#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;

    use aws_sdk_dynamodb::{Client, Credentials, Region};
    use cqrs_es::{
        Aggregate, AggregateError, DomainEvent, EventEnvelope, EventStore, UserErrorPayload, View,
    };
    use persist_es::{
        GenericQuery, PersistedEventStore, PersistedSnapshotStore, SerializedEvent,
        SerializedSnapshot,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    use crate::{DynamoEventRepository, DynamoViewRepository};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub(crate) struct TestAggregate {
        pub(crate) id: String,
        pub(crate) description: String,
        pub(crate) tests: Vec<String>,
    }

    impl Aggregate for TestAggregate {
        type Command = TestCommand;
        type Event = TestEvent;
        type Error = UserErrorPayload;

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn handle(
            &self,
            _command: Self::Command,
        ) -> Result<Vec<Self::Event>, AggregateError<UserErrorPayload>> {
            Ok(vec![])
        }

        fn apply(&mut self, _e: Self::Event) {}
    }

    impl Default for TestAggregate {
        fn default() -> Self {
            TestAggregate {
                id: "".to_string(),
                description: "".to_string(),
                tests: Vec::new(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub enum TestEvent {
        Created(Created),
        Tested(Tested),
        SomethingElse(SomethingElse),
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Created {
        pub id: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct Tested {
        pub test_name: String,
    }

    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct SomethingElse {
        pub description: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            match self {
                TestEvent::Created(_) => "Created",
                TestEvent::Tested(_) => "Tested",
                TestEvent::SomethingElse(_) => "SomethingElse",
            }
        }

        fn event_version(&self) -> &'static str {
            "1.0"
        }
    }

    pub enum TestCommand {}

    pub(crate) type TestQueryRepository =
        GenericQuery<DynamoViewRepository<TestView, TestAggregate>, TestView, TestAggregate>;

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub(crate) struct TestView {
        events: Vec<TestEvent>,
    }

    impl View<TestAggregate> for TestView {
        fn update(&mut self, event: &EventEnvelope<TestAggregate>) {
            self.events.push(event.payload.clone());
        }
    }

    pub async fn test_dynamodb_client() -> Client {
        let endpoint_uri = "http://localhost:8000".try_into().unwrap();
        let endpoint = aws_sdk_dynamodb::Endpoint::immutable(endpoint_uri);
        let region = Region::new("us-west-2");
        let credentials = Credentials::new("", "", None, None, "");
        let config = aws_sdk_dynamodb::config::Config::builder()
            .region(region)
            .endpoint_resolver(endpoint)
            .credentials_provider(credentials)
            .build();
        aws_sdk_dynamodb::client::Client::from_conf(config)
    }

    pub(crate) async fn new_test_event_store(
        client: Client,
    ) -> PersistedEventStore<DynamoEventRepository, TestAggregate> {
        let repo = DynamoEventRepository::new(client);
        PersistedEventStore::<DynamoEventRepository, TestAggregate>::new(repo)
    }

    pub(crate) async fn new_test_snapshot_store(
        client: Client,
    ) -> PersistedSnapshotStore<DynamoEventRepository, TestAggregate> {
        let repo = DynamoEventRepository::new(client);
        PersistedSnapshotStore::<DynamoEventRepository, TestAggregate>::new(repo)
    }

    pub(crate) fn new_test_metadata() -> HashMap<String, String> {
        let now = "2021-03-18T12:32:45.930Z".to_string();
        let mut metadata = HashMap::new();
        metadata.insert("time".to_string(), now);
        metadata
    }

    pub(crate) fn test_event_envelope(
        id: &str,
        sequence: usize,
        event: TestEvent,
    ) -> SerializedEvent {
        let payload: Value = serde_json::to_value(&event).unwrap();
        SerializedEvent {
            aggregate_id: id.to_string(),
            sequence,
            aggregate_type: TestAggregate::aggregate_type().to_string(),
            event_type: event.event_type().to_string(),
            event_version: event.event_version().to_string(),
            payload,
            metadata: Default::default(),
        }
    }
    pub(crate) fn snapshot_context(
        aggregate_id: String,
        current_sequence: usize,
        current_snapshot: usize,
        aggregate: Value,
    ) -> SerializedSnapshot {
        SerializedSnapshot {
            aggregate_id,
            aggregate,
            current_sequence,
            current_snapshot,
        }
    }

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

    // #[tokio::test]
    // async fn event_repositories() {
    //     let client = test_dynamodb_client().await;
    //     let id = uuid::Uuid::new_v4().to_string();
    //     let event_repo = DynamoEventRepository::new(client.clone());
    //     let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
    //     assert!(events.is_empty());
    //
    //     event_repo
    //         .insert_events::<TestAggregate>(&[
    //             test_event_envelope(&id, 1, TestEvent::Created(Created { id: id.clone() })),
    //             test_event_envelope(
    //                 &id,
    //                 2,
    //                 TestEvent::Tested(Tested {
    //                     test_name: "a test was run".to_string(),
    //                 }),
    //             ),
    //         ])
    //         .await
    //         .unwrap();
    //     let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
    //     assert_eq!(2, events.len());
    //     events.iter().for_each(|e| assert_eq!(&id, &e.aggregate_id));
    //
    //     event_repo
    //         .insert_events::<TestAggregate>(&[
    //             test_event_envelope(
    //                 &id,
    //                 3,
    //                 TestEvent::SomethingElse(SomethingElse {
    //                     description: "this should not persist".to_string(),
    //                 }),
    //             ),
    //             test_event_envelope(
    //                 &id,
    //                 2,
    //                 TestEvent::SomethingElse(SomethingElse {
    //                     description: "bad sequence number".to_string(),
    //                 }),
    //             ),
    //         ])
    //         .await
    //         .unwrap_err();
    //     let events = event_repo.get_events::<TestAggregate>(&id).await.unwrap();
    //     assert_eq!(2, events.len());
    //
    // }
}
