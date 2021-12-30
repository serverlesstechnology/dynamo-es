#[cfg(test)]
mod tests {
    use cqrs_es::{Aggregate, AggregateError, DomainEvent, EventEnvelope, View};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub struct TestAggregate {
        id: String,
        description: String,
        tests: Vec<String>,
    }

    impl Aggregate for TestAggregate {
        type Command = TestCommand;
        type Event = TestEvent;

        fn aggregate_type() -> &'static str {
            "TestAggregate"
        }

        fn handle(&self, _command: Self::Command) -> Result<Vec<Self::Event>, AggregateError> {
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

    #[derive(Debug, Default, Serialize, Deserialize)]
    struct TestView {
        events: Vec<TestEvent>,
    }

    impl View<TestAggregate> for TestView {
        fn update(&mut self, event: &EventEnvelope<TestAggregate>) {
            self.events.push(event.payload.clone());
        }
    }

    #[tokio::test]
    async fn test_valid_cqrs_framework() {}

    #[tokio::test]
    async fn commit_and_load_events() {}

    #[tokio::test]
    async fn commit_and_load_events_snapshot_store() {}

    #[test]
    fn test_event_breakout_type() {}

    #[tokio::test]
    async fn event_repositories() {}

    #[tokio::test]
    async fn snapshot_repositories() {}
}
